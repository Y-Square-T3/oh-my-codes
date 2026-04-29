import { Effect, Layer, Option, Context } from "effect"

import * as Account from "../account"
import * as Repo from "./repo"
import { fetchApiJson, ModelApiError } from "./api"
import { transformProvider, transformModel } from "./transformer"
import { type ModelRow } from "./schema"
import { type AccountID } from "../account/schema"

export class ModelServiceError extends Error {
  readonly _tag = "ModelServiceError"
  constructor(readonly message: string, readonly cause?: unknown) {
    super(message)
    this.name = "ModelServiceError"
  }
}

export interface ModelInfo {
  id: string
  providerId: string
  name: string
  family: string | null
  reasoning: boolean | null
  toolCall: boolean | null
  attachment: boolean | null
  temperature: boolean | null
  openWeights: boolean | null
  modalitiesInput: string[]
  modalitiesOutput: string[]
  costInput: number
  costOutput: number
  limitContext: number | null
  limitOutput: number | null
  releaseDate: string | null
}

export interface ProviderInfo {
  id: string
  name: string
  api: string | null
  npm: string | null
  modelCount: number
}

export interface RefreshResult {
  providers: number
  models: number
}

export interface ListResult {
  providers: ProviderInfo[]
  models: ModelInfo[]
  accountEmail: string | null
  accountUrl: string | null
}

export interface ClearResult {
  modelsDeleted: number
  providersDeleted: number
}

export interface ModelServiceInterface {
  readonly list: (providerId?: string) => Effect.Effect<ListResult, ModelServiceError>
  readonly refresh: () => Effect.Effect<RefreshResult, ModelServiceError | ModelApiError>
  readonly clear: (providerId?: string) => Effect.Effect<ClearResult, ModelServiceError>
}

export const Service = Context.GenericTag<ModelServiceInterface>("@model/Model")

const parseModalities = (input: string | null): string[] => {
  if (!input) return []
  try { return JSON.parse(input) } catch { return [] }
}

const fromDbBool = (v: number | null): boolean | null => v === null ? null : v === 1

const mapModelRow = (row: ModelRow, providerId: string): ModelInfo => ({
  id: row.id,
  providerId,
  name: row.name,
  family: row.family,
  reasoning: fromDbBool(row.reasoning),
  toolCall: fromDbBool(row.toolCall),
  attachment: fromDbBool(row.attachment),
  temperature: fromDbBool(row.temperature),
  openWeights: fromDbBool(row.openWeights),
  modalitiesInput: parseModalities(row.modalitiesInput),
  modalitiesOutput: parseModalities(row.modalitiesOutput),
  costInput: row.costInput ?? 0,
  costOutput: row.costOutput ?? 0,
  limitContext: row.limitContext ?? null,
  limitOutput: row.limitOutput ?? null,
  releaseDate: row.releaseDate ?? null,
})

const liftError = <A, E>(effect: Effect.Effect<A, E>): Effect.Effect<A, ModelServiceError> =>
  effect.pipe(
    Effect.catchAll((e: E) =>
      Effect.fail(new ModelServiceError(String(e), e))
    ),
  )

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const repo = yield* Repo.Service
    const accountSvc = yield* Account.Service

    const list = (providerId?: string) =>
      Effect.gen(function* () {
        const accountOpt = yield* liftError(
          accountSvc.active().pipe(
            Effect.catchAll(() => Effect.succeed(Option.none())),
          ),
        )

        if (Option.isNone(accountOpt)) {
          return {
            providers: [] as ProviderInfo[],
            models: [] as ModelInfo[],
            accountEmail: null,
            accountUrl: null,
          } satisfies ListResult
        }

        const account = Option.getOrThrow(accountOpt)
        const accountId = account.id as AccountID

        const providers = yield* liftError(
          providerId
            ? repo.listProviders(accountId).pipe(
                Effect.map((ps) => ps.filter((p) => p.id === providerId)),
              )
            : repo.listProviders(accountId),
        )

        const providerInfos: ProviderInfo[] = providers.map((p) => ({
          id: p.id,
          name: p.name,
          api: p.api,
          npm: p.npm,
          modelCount: 0,
        }))

        const allModels: ModelInfo[] = []
        for (const p of providers) {
          const models = yield* liftError(repo.listModels({ providerId: p.id, accountId }))
          for (const m of models) {
            allModels.push(mapModelRow(m, p.id))
          }
        }

        for (const pi of providerInfos) {
          pi.modelCount = allModels.filter((m) => m.providerId === pi.id).length
        }

        return {
          providers: providerInfos,
          models: allModels,
          accountEmail: account.email,
          accountUrl: account.url,
        } satisfies ListResult
      })

    const refresh = () =>
      Effect.gen(function* () {
        const accountOpt = yield* liftError(
          accountSvc.active().pipe(
            Effect.catchAll(() => Effect.succeed(Option.none())),
          ),
        )

        if (Option.isNone(accountOpt)) {
          return yield* Effect.fail(new ModelServiceError("No account logged in. Run `account login` first."))
        }

        const account = Option.getOrThrow(accountOpt)
        const accountId = account.id as AccountID

        const tokenOpt = yield* liftError(
          accountSvc.token(accountId).pipe(
            Effect.catchAll(() => Effect.succeed(Option.none())),
          ),
        )

        if (Option.isNone(tokenOpt)) {
          return yield* Effect.fail(new ModelServiceError("Failed to get access token"))
        }

        const token = Option.getOrThrow(tokenOpt)

        const apiData = yield* Effect.tryPromise({
          try: () => fetchApiJson(account.url, accountId, token),
          catch: (e) => e instanceof ModelApiError
            ? e
            : new ModelApiError("GET", `${account.url}/models/api.json`, String(e)),
        })

        const transformedProviders = Object.entries(apiData).map(([id, p]) =>
          transformProvider(id, p, accountId)
        )

        const transformedModels: ReturnType<typeof transformModel>[] = []
        for (const [providerId, provider] of Object.entries(apiData)) {
          for (const [modelId, model] of Object.entries(provider.models)) {
            transformedModels.push(transformModel(providerId, modelId, model, accountId))
          }
        }

        yield* liftError(repo.upsertProviders(transformedProviders))
        yield* liftError(repo.upsertModels(transformedModels))

        return {
          providers: transformedProviders.length,
          models: transformedModels.length,
        } satisfies RefreshResult
      })

    const clear = (providerId?: string) =>
      Effect.gen(function* () {
        const accountOpt = yield* liftError(
          accountSvc.active().pipe(
            Effect.catchAll(() => Effect.succeed(Option.none())),
          ),
        )

        if (Option.isNone(accountOpt)) {
          return yield* Effect.fail(new ModelServiceError("No account logged in"))
        }

        const account = Option.getOrThrow(accountOpt)
        const accountId = account.id as AccountID

        const existingModels = yield* liftError(repo.listModels({ accountId }))
        const filteredModels = providerId
          ? existingModels.filter((m) => m.providerId === providerId)
          : existingModels

        yield* liftError(repo.deleteByAccountId(accountId))

        if (!providerId) {
          yield* liftError(repo.deleteProviderByAccountId(accountId))
          return { modelsDeleted: filteredModels.length, providersDeleted: 0 } satisfies ClearResult
        }

        const remainingModels = yield* liftError(repo.listModels({ accountId }))
        const providersToCheck = yield* liftError(repo.listProviders(accountId))
        const providersWithModels = new Set(remainingModels.map((m) => m.providerId))
        const providersToDelete = providersToCheck.filter((p) => !providersWithModels.has(p.id))
        for (const _p of providersToDelete) {
          yield* liftError(repo.deleteProviderByAccountId(accountId))
        }

        return {
          modelsDeleted: filteredModels.length,
          providersDeleted: providersToDelete.length,
        } satisfies ClearResult
      })

    return {
      list,
      refresh,
      clear,
    } satisfies ModelServiceInterface
  }),
)

export const defaultLayer = layer.pipe(
  Layer.provide(Repo.defaultLayer),
  Layer.provide(Account.defaultLayer),
)
