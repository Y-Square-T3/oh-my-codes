import { Context, Effect, Layer } from "effect"
import { and, eq } from "drizzle-orm"

import * as Db from "../database"
import { modelRecords, type ModelRow, type ProviderRow, providers } from "./schema"
import { type TransformedModel, type TransformedProvider } from "./transformer"
import { type AccountID } from "../account/schema"

export class ModelRepoError extends Error {
  readonly _tag = "ModelRepoError"
  constructor(
    readonly message: string,
    readonly cause?: unknown,
  ) {
    super(message)
    this.name = "ModelRepoError"
  }
}

export interface ModelRepoService {
  readonly listProviders: (accountId?: AccountID) => Effect.Effect<ProviderRow[], ModelRepoError>
  readonly listModels: (opts?: {
    providerId?: string
    accountId?: AccountID
  }) => Effect.Effect<ModelRow[], ModelRepoError>
  readonly upsertProviders: (items: TransformedProvider[]) => Effect.Effect<void, ModelRepoError>
  readonly upsertModels: (items: TransformedModel[]) => Effect.Effect<void, ModelRepoError>
  readonly deleteByAccountId: (accountId: AccountID) => Effect.Effect<void, ModelRepoError>
  readonly deleteProviderByAccountId: (accountId: AccountID) => Effect.Effect<void, ModelRepoError>
}

export const Service = Context.GenericTag<ModelRepoService>("@model/ModelRepo")

const toDbBool = (v: boolean | null): number | null => (v === null ? null : v ? 1 : 0)

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const database = yield* Db.Database
    yield* database.migrate()

    const listProviders = (accountId?: AccountID) =>
      Effect.tryPromise({
        try: async () => {
          if (accountId) {
            return database.db.select().from(providers).where(eq(providers.accountId, accountId)).all()
          }
          return database.db.select().from(providers).all()
        },
        catch: (cause) => new Db.DatabaseQueryError("Failed to list providers", cause),
      }).pipe(
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to list providers", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error listing providers", cause)),
        ),
      )

    const listModels = (opts?: { providerId?: string; accountId?: AccountID }) =>
      Effect.tryPromise({
        try: async () => {
          if (opts?.providerId && opts?.accountId) {
            return database.db
              .select()
              .from(modelRecords)
              .where(and(eq(modelRecords.providerId, opts.providerId), eq(modelRecords.accountId, opts.accountId)))
              .all()
          }
          if (opts?.providerId) {
            return database.db.select().from(modelRecords).where(eq(modelRecords.providerId, opts.providerId)).all()
          }
          if (opts?.accountId) {
            return database.db.select().from(modelRecords).where(eq(modelRecords.accountId, opts.accountId)).all()
          }
          return database.db.select().from(modelRecords).all()
        },
        catch: (cause) => new Db.DatabaseQueryError("Failed to list models", cause),
      }).pipe(
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to list models", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error listing models", cause)),
        ),
      )

    const upsertProviders = (items: TransformedProvider[]) =>
      Effect.tryPromise({
        try: async () => {
          for (const p of items) {
            await database.db
              .insert(providers)
              .values({
                id: p.id,
                name: p.name,
                api: p.api,
                npm: p.npm,
                doc: p.doc,
                envVars: p.envVars,
                accountId: p.accountId,
                lastFetchedAt: p.lastFetchedAt,
                createdAt: p.createdAt,
                updatedAt: p.updatedAt,
              })
              .onConflictDoUpdate({
                target: [providers.id, providers.accountId],
                set: {
                  name: p.name,
                  api: p.api,
                  npm: p.npm,
                  doc: p.doc,
                  envVars: p.envVars,
                  lastFetchedAt: p.lastFetchedAt,
                  updatedAt: p.updatedAt,
                },
              })
          }
        },
        catch: (cause) => {
          return new Db.DatabaseQueryError("Failed to upsert providers", cause)
        },
      }).pipe(
        Effect.asVoid,
        Effect.tap(() => database.flush()),
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to upsert providers", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error upserting providers", cause)),
        ),
      )

    const upsertModels = (items: TransformedModel[]) =>
      Effect.tryPromise({
        try: async () => {
          for (const m of items) {
            await database.db
              .insert(modelRecords)
              .values({
                id: m.id,
                providerId: m.providerId,
                name: m.name,
                family: m.family,
                attachment: toDbBool(m.attachment),
                reasoning: toDbBool(m.reasoning),
                toolCall: toDbBool(m.toolCall),
                enable: toDbBool(m.enable),
                structuredOutput: toDbBool(m.structuredOutput),
                temperature: toDbBool(m.temperature),
                interleavedField: m.interleavedField,
                knowledge: m.knowledge,
                releaseDate: m.releaseDate,
                lastUpdated: m.lastUpdated,
                openWeights: toDbBool(m.openWeights),
                modalitiesInput: m.modalitiesInput,
                modalitiesOutput: m.modalitiesOutput,
                costInput: m.costInput,
                costOutput: m.costOutput,
                costReasoning: m.costReasoning,
                costCacheRead: m.costCacheRead,
                costCacheWrite: m.costCacheWrite,
                limitContext: m.limitContext,
                limitOutput: m.limitOutput,
                accountId: m.accountId,
                createdAt: m.createdAt,
                updatedAt: m.updatedAt,
              })
              .onConflictDoUpdate({
                target: [modelRecords.id, modelRecords.providerId, modelRecords.accountId],
                set: {
                  name: m.name,
                  family: m.family,
                  attachment: toDbBool(m.attachment),
                  reasoning: toDbBool(m.reasoning),
                  toolCall: toDbBool(m.toolCall),
                  enable: toDbBool(m.enable),
                  structuredOutput: toDbBool(m.structuredOutput),
                  temperature: toDbBool(m.temperature),
                  interleavedField: m.interleavedField,
                  knowledge: m.knowledge,
                  releaseDate: m.releaseDate,
                  lastUpdated: m.lastUpdated,
                  openWeights: toDbBool(m.openWeights),
                  modalitiesInput: m.modalitiesInput,
                  modalitiesOutput: m.modalitiesOutput,
                  costInput: m.costInput,
                  costOutput: m.costOutput,
                  costReasoning: m.costReasoning,
                  costCacheRead: m.costCacheRead,
                  costCacheWrite: m.costCacheWrite,
                  limitContext: m.limitContext,
                  limitOutput: m.limitOutput,
                  updatedAt: m.updatedAt,
                },
              })
          }
        },
        catch: (cause) => {
          return new Db.DatabaseQueryError("Failed to upsert models", cause)
        },
      }).pipe(
        Effect.asVoid,
        Effect.tap(() => database.flush()),
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to upsert models", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error upserting models", cause)),
        ),
      )

    const deleteByAccountId = (accountId: AccountID) =>
      Effect.tryPromise({
        try: async () => {
          await database.db.delete(modelRecords).where(eq(modelRecords.accountId, accountId)).run()
        },
        catch: (cause) => new Db.DatabaseQueryError("Failed to delete models", cause),
      }).pipe(
        Effect.asVoid,
        Effect.tap(() => database.flush()),
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to delete models", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error deleting models", cause)),
        ),
      )

    const deleteProviderByAccountId = (accountId: AccountID) =>
      Effect.tryPromise({
        try: async () => {
          await database.db.delete(providers).where(eq(providers.accountId, accountId)).run()
        },
        catch: (cause) => new Db.DatabaseQueryError("Failed to delete providers", cause),
      }).pipe(
        Effect.asVoid,
        Effect.tap(() => database.flush()),
        Effect.catchAll((cause: unknown) =>
          cause instanceof Db.DatabaseQueryError
            ? Effect.fail(new ModelRepoError("Failed to delete providers", cause.cause))
            : Effect.fail(new ModelRepoError("Unexpected error deleting providers", cause)),
        ),
      )

    return {
      listProviders,
      listModels,
      upsertProviders,
      upsertModels,
      deleteByAccountId,
      deleteProviderByAccountId,
    } satisfies ModelRepoService
  }),
)

export const defaultLayer = layer.pipe(Layer.provide(Db.defaultLayer))
