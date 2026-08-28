import type { Plugin } from "@opencode-ai/plugin"
import type {
  DaemonCredentialsResponse,
  DaemonModelInfo,
  DaemonModelsListResponse,
  DaemonProviderInfo,
  TokenUsageRecord,
  V1MessageUpdatedEvent,
  V2StepEndedEvent,
} from "../types.js"

const COMPACTION_AGENTS = ["compaction", "summarize"]

function isCompactionAgent(agent?: string): boolean {
  if (!agent) return false
  const lower = agent.toLowerCase()
  return COMPACTION_AGENTS.some((a) => lower.includes(a))
}

function getDaemonUrl(): string {
  return process.env.OMC_DAEMON_URL ?? "http://127.0.0.1:9823"
}

async function sendToDaemon(
  url: string,
  record: TokenUsageRecord,
  log: (msg: string) => Promise<void>,
): Promise<void> {
  try {
    const res = await fetch(`${url}/token-usage`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(record),
    })
    if (!res.ok) {
      await log(`omcd responded ${res.status} for token-usage`)
    }
  } catch (e) {
    await log(`Failed to send token-usage to omcd: ${e}`)
  }
}

function extractV1(event: V1MessageUpdatedEvent): TokenUsageRecord | null {
  const info = event.properties?.info
  if (!info) return null
  if (info.role !== "assistant" || !info.finish) return null
  if (!info.sessionID || !info.tokens) return null
  if (isCompactionAgent(info.agent)) return null

  return {
    sessionId: info.sessionID,
    messageId: info.id ?? info.messageID ?? "",
    agent: info.agent ?? "",
    model: info.modelID ?? "unknown",
    inputTokens: info.tokens.input ?? 0,
    outputTokens: info.tokens.output ?? 0,
    reasoningTokens: info.tokens.reasoning ?? 0,
    cacheReadTokens: info.tokens.cache?.read ?? 0,
    cacheWriteTokens: info.tokens.cache?.write ?? 0,
    recordedAt: Date.now(),
  }
}

function extractV2(event: V2StepEndedEvent): TokenUsageRecord | null {
  const p = event.properties
  if (!p) return null
  if (!p.sessionID || !p.tokens) return null
  if (isCompactionAgent(p.agent)) return null

  return {
    sessionId: p.sessionID,
    messageId: p.assistantMessageID ?? "",
    agent: p.agent ?? "",
    model: p.modelID ?? "unknown",
    inputTokens: p.tokens.input ?? 0,
    outputTokens: p.tokens.output ?? 0,
    reasoningTokens: p.tokens.reasoning ?? 0,
    cacheReadTokens: p.tokens.cache?.read ?? 0,
    cacheWriteTokens: p.tokens.cache?.write ?? 0,
    recordedAt: Date.now(),
  }
}

async function fetchModels(
  daemonUrl: string,
): Promise<DaemonModelsListResponse | null> {
  try {
    const res = await fetch(`${daemonUrl}/models`)
    if (!res.ok) return null
    return (await res.json()) as DaemonModelsListResponse
  } catch {
    return null
  }
}

async function fetchCredentials(
  daemonUrl: string,
): Promise<DaemonCredentialsResponse | null> {
  try {
    const res = await fetch(`${daemonUrl}/account/credentials`)
    if (!res.ok) return null
    return (await res.json()) as DaemonCredentialsResponse
  } catch {
    return null
  }
}

function buildModelConfig(model: DaemonModelInfo): Record<string, unknown> {
  const config: Record<string, unknown> = {}

  if (model.limitContext != null || model.limitOutput != null) {
    config.limit = {
      context: model.limitContext ?? 0,
      output: model.limitOutput ?? 0,
    }
  }

  if (model.modalitiesInput.length > 0 || model.modalitiesOutput.length > 0) {
    const modalities: Record<string, string[]> = {}
    if (model.modalitiesInput.length > 0)
      modalities.input = model.modalitiesInput
    if (model.modalitiesOutput.length > 0)
      modalities.output = model.modalitiesOutput
    config.modalities = modalities
  }

  const capabilities: Record<string, unknown> = {}
  if (model.attachment) capabilities.attachment = true
  if (model.reasoning) capabilities.reasoning = true
  if (model.toolCall) capabilities.tool_call = true
  if (model.temperature) capabilities.temperature = true
  if (model.openWeights) capabilities.open_weights = true
  if (model.modalitiesInput.includes("image")) {
    capabilities.input = { image: true }
  }
  if (Object.keys(capabilities).length > 0) {
    config.capabilities = capabilities
  }

  return config
}

function buildProviderConfig(
  providers: DaemonProviderInfo[],
  models: DaemonModelInfo[],
  credentials: DaemonCredentialsResponse,
): Record<string, unknown> {
  const modelsByProvider = new Map<string, DaemonModelInfo[]>()
  for (const model of models) {
    const existing = modelsByProvider.get(model.providerId) ?? []
    existing.push(model)
    modelsByProvider.set(model.providerId, existing)
  }

  const providerConfig: Record<string, unknown> = {}

  for (const provider of providers) {
    const providerModels = modelsByProvider.get(provider.id) ?? []
    const modelConfigs: Record<string, unknown> = {}
    for (const model of providerModels) {
      modelConfigs[model.id] = buildModelConfig(model)
    }

    const providerEntry: Record<string, unknown> = {
      models: modelConfigs,
    }

    const options: Record<string, unknown> = {
      apiKey: credentials.apiKey,
      baseURL: credentials.baseUrl,
    }
    if (credentials.workspaceId) {
      options.headers = {
        "x-workspace-id": credentials.workspaceId,
      }
    }
    providerEntry.options = options

    providerConfig[provider.id] = providerEntry
  }

  return providerConfig
}

export const OhMyCodesPlugin: Plugin = async ({ client }) => {
  const daemonUrl = getDaemonUrl()
  const log = async (msg: string) => {
    await client.app.log({
      body: {
        service: "oh-my-codes",
        level: "info",
        message: msg,
      },
    })
  }

  return {
    config: async (config) => {
      const [modelsData, credentials] = await Promise.all([
        fetchModels(daemonUrl),
        fetchCredentials(daemonUrl),
      ])

      if (!modelsData || !credentials) {
        await log(
          "Failed to fetch models or credentials from omcd, skipping provider injection",
        )
        return
      }

      if (modelsData.providers.length === 0) {
        await log("No models available from omcd, skipping provider injection")
        return
      }

      const injectedProviders = buildProviderConfig(
        modelsData.providers,
        modelsData.models,
        credentials,
      )

      const configAny = config as Record<string, unknown>
      const existingProviders =
        (configAny.provider as Record<string, unknown>) ?? {}
      configAny.provider = { ...injectedProviders, ...existingProviders }

      await log(
        `Injected ${modelsData.providers.length} providers with ${modelsData.models.length} models from omcd`,
      )
    },

    event: async ({ event }) => {
      const e = event as { type: string; properties?: unknown }

      let record: TokenUsageRecord | null = null

      if (e.type === "message.updated") {
        record = extractV1(e as unknown as V1MessageUpdatedEvent)
      } else if (e.type === "session.next.step.ended") {
        record = extractV2(e as unknown as V2StepEndedEvent)
      }

      if (record) {
        await sendToDaemon(daemonUrl, record, log)
      }
    },
  }
}
