import { Cause, Effect, Layer, Option } from "effect"
import * as Account from "../features/account"
import { modelRepoDefaultLayer, ModelRepoService } from "../features/model"
import { log } from "../shared"
import type { AccountID } from "../features/account/schema"
import type { ModelRow, ProviderRow } from "../features/model/schema"

type AccountProviderConfigDeps = {
  config: Record<string, unknown>
  layer?: Layer.Layer<never, never>
}

type AccountCredentials = {
  apiKey: string
  baseURL: string
  workspaceId: string | null
}

function parseJsonArray(value: string | null): string[] | null {
  if (!value) return null
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

function buildModelConfig(row: ModelRow): Record<string, unknown> {
  const modelConfig: Record<string, unknown> = {}

  if (row.limitContext != null) {
    modelConfig.limit = { context: row.limitContext }
  }

  const inputModalities = parseJsonArray(row.modalitiesInput)
  const outputModalities = parseJsonArray(row.modalitiesOutput)
  if (inputModalities || outputModalities) {
    const modalities: Record<string, string[] | undefined> = {}
    if (inputModalities) modalities.input = inputModalities
    if (outputModalities) modalities.output = outputModalities
    modelConfig.modalities = modalities
  }

  const capabilities: Record<string, unknown> = {}
  if (row.attachment === 1) capabilities.attachment = true
  if (row.reasoning === 1) capabilities.reasoning = true
  if (row.toolCall === 1) capabilities.tool_call = true
  if (row.structuredOutput === 1) capabilities.structured_output = true
  if (row.temperature === 1) capabilities.temperature = true
  if (row.openWeights === 1) capabilities.open_weights = true

  if (inputModalities?.includes("image")) {
    capabilities.input = { image: true }
  }

  if (Object.keys(capabilities).length > 0) {
    modelConfig.capabilities = capabilities
  }

  if (row.interleavedField) {
    modelConfig.interleaved = { field: row.interleavedField }
  }

  return modelConfig
}

export function buildAccountProviderConfig(
  providers: ProviderRow[],
  models: ModelRow[],
  credentials: AccountCredentials,
): Record<string, unknown> {
  const modelsByProvider = new Map<string, ModelRow[]>()
  for (const model of models) {
    const existing = modelsByProvider.get(model.providerId) || []
    existing.push(model)
    modelsByProvider.set(model.providerId, existing)
  }

  const accountProviders: Record<string, unknown> = {}
  for (const provider of providers) {
    const providerModels = modelsByProvider.get(provider.id) || []
    const modelConfigs: Record<string, unknown> = {}

    for (const model of providerModels) {
      modelConfigs[model.id] = buildModelConfig(model)
    }

    const providerEntry: Record<string, unknown> = {
      models: modelConfigs,
    }

    const options: Record<string, unknown> = {}
    options.apiKey = credentials.apiKey
    if (credentials.baseURL) {
      options.baseURL = credentials.baseURL
    }
    if (credentials.workspaceId) {
      options.headers = {
        "x-workspace-id": credentials.workspaceId,
      }
    }
    providerEntry.options = options

    accountProviders[provider.id] = providerEntry
  }

  return accountProviders
}

export async function applyAccountProviderConfig(
  deps: AccountProviderConfigDeps,
): Promise<void> {
  const effectiveLayer =
    deps.layer ?? Layer.mergeAll(Account.defaultLayer, modelRepoDefaultLayer)

  const result = await Effect.runPromiseExit(
    (doApplyAccountProviderConfig(deps) as any).pipe(
      Effect.provide(effectiveLayer),
    ),
  )

  if (result._tag === "Failure") {
    log(`Account provider config failed: ${Cause.pretty(result.cause)}`)
  }
}

function doApplyAccountProviderConfig(deps: AccountProviderConfigDeps) {
  return Effect.gen(function* () {
    const accountSvc = yield* Account.Service
    const accountOpt = yield* accountSvc.activeWithToken()

    if (Option.isNone(accountOpt)) {
      log("No active account, skipping account provider config")
      return
    }

    const { account, accessToken } = accountOpt.value
    const modelRepo = yield* ModelRepoService
    const providers = yield* modelRepo.listProviders(account.id as AccountID)

    if (providers.length === 0) {
      log(`No providers found for account ${account.id}`)
      return
    }

    const models = yield* modelRepo.listModels({
      accountId: account.id as AccountID,
    })

    const providerConfig = buildAccountProviderConfig(providers, models, {
      apiKey: accessToken,
      baseURL: `${account.url.replace(/\/$/, "")}/api/v2`,
      workspaceId: account.activeWorkspaceId,
    })

    const existing = deps.config.provider as Record<string, unknown> | undefined
    deps.config.provider = { ...providerConfig, ...existing }

    log(
      `Applied account provider config: ${providers.length} providers, ${models.length} models for account ${account.id}`,
    )
  })
}
