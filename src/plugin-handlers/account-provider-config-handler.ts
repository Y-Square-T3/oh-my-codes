import { Database } from "bun:sqlite"
import { resolveDbPath } from "../features/database/database"
import { log } from "../shared"

type AccountProviderConfigDeps = {
  config: Record<string, unknown>
}

type ModelRow = {
  id: string
  provider_id: string
  limit_context: number | null
  modalities_input: string | null
  modalities_output: string | null
  attachment: number | null
  reasoning: number | null
  tool_call: number | null
  structured_output: number | null
  temperature: number | null
  open_weights: number | null
  interleaved_field: string | null
  family: string | null
  knowledge: string | null
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

  if (row.limit_context != null) {
    modelConfig.limit = { context: row.limit_context }
  }

  const inputModalities = parseJsonArray(row.modalities_input)
  const outputModalities = parseJsonArray(row.modalities_output)
  if (inputModalities || outputModalities) {
    const modalities: Record<string, string[] | undefined> = {}
    if (inputModalities) modalities.input = inputModalities
    if (outputModalities) modalities.output = outputModalities
    modelConfig.modalities = modalities
  }

  const capabilities: Record<string, unknown> = {}
  if (row.attachment === 1) capabilities.attachment = true
  if (row.reasoning === 1) capabilities.reasoning = true
  if (row.tool_call === 1) capabilities.tool_call = true
  if (row.structured_output === 1) capabilities.structured_output = true
  if (row.temperature === 1) capabilities.temperature = true
  if (row.open_weights === 1) capabilities.open_weights = true

  if (inputModalities?.includes("image")) {
    capabilities.input = { image: true }
  }

  if (Object.keys(capabilities).length > 0) {
    modelConfig.capabilities = capabilities
  }

  if (row.interleaved_field) {
    modelConfig.interleaved = { field: row.interleaved_field }
  }

  return modelConfig
}

export function applyAccountProviderConfig(
  deps: AccountProviderConfigDeps,
): void {
  const dbPath = resolveDbPath()
  const db = new Database(dbPath)

  try {
    const accountState = db
      .prepare(
        "SELECT active_account_id FROM account_state WHERE id = 1",
      )
      .get() as { active_account_id: string | null } | undefined

    if (!accountState?.active_account_id) {
      log("No active account, skipping account provider config")
      return
    }

    const accountId = accountState.active_account_id

    const providers = db
      .prepare("SELECT id, name FROM providers WHERE account_id = ?")
      .all(accountId) as Array<{ id: string; name: string }>

    if (!providers.length) {
      log(`No providers found for account ${accountId}`)
      return
    }

    const models = db
      .prepare(
        "SELECT id, provider_id, limit_context, modalities_input, " +
          "modalities_output, attachment, reasoning, tool_call, " +
          "structured_output, temperature, open_weights, interleaved_field " +
          "FROM models WHERE account_id = ?",
      )
      .all(accountId) as ModelRow[]

    const modelsByProvider = new Map<string, ModelRow[]>()
    for (const model of models) {
      const existing = modelsByProvider.get(model.provider_id) || []
      existing.push(model)
      modelsByProvider.set(model.provider_id, existing)
    }

    const accountProviders: Record<string, unknown> = {}
    for (const provider of providers) {
      const providerModels = modelsByProvider.get(provider.id) || []
      const modelConfigs: Record<string, unknown> = {}

      for (const model of providerModels) {
        modelConfigs[model.id] = buildModelConfig(model)
      }

      accountProviders[provider.id] = {
        models: modelConfigs,
      }
    }

    const existingProviders = deps.config.provider as
      | Record<string, unknown>
      | undefined
    deps.config.provider = {
      ...accountProviders,
      ...existingProviders,
    }

    log(
      `Applied account provider config: ${providers.length} providers, ${models.length} models for account ${accountId}`,
    )
  } finally {
    db.close()
  }
}