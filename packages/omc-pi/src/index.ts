import type {
  ExtensionAPI,
  ExtensionContext,
  MessageEndEvent,
  ProviderConfig,
  ProviderModelConfig,
} from "@oh-my-pi/pi-coding-agent/extensibility/extensions"
import type {
  DaemonCredentialsResponse,
  DaemonModelInfo,
  DaemonModelsListResponse,
  DaemonProviderInfo,
} from "./types.js"

interface TokenUsageRecord {
  sessionId: string
  messageId: string
  agent: string
  model: string
  inputTokens: number
  outputTokens: number
  reasoningTokens: number
  cacheReadTokens: number
  cacheWriteTokens: number
  recordedAt: number
}

function getDaemonUrl(): string {
  return process.env.OMC_DAEMON_URL ?? "http://127.0.0.1:9823"
}

async function sendToDaemon(
  url: string,
  record: TokenUsageRecord,
  logger: ExtensionAPI["logger"],
): Promise<void> {
  try {
    const res = await fetch(`${url}/token-usage`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(record),
    })
    if (!res.ok) {
      logger.warn(`omcd responded ${res.status} for token-usage`)
    }
  } catch (e) {
    logger.warn(`Failed to send token-usage to omcd: ${e}`)
  }
}

function generateMessageId(): string {
  return `msg-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
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

function transformModel(model: DaemonModelInfo): ProviderModelConfig {
  const input = model.modalitiesInput.filter(
    (m): m is "text" | "image" => m === "text" || m === "image",
  )

  return {
    id: model.id,
    name: model.name,
    reasoning: model.reasoning ?? false,
    input,
    cost: {
      input: model.costInput,
      output: model.costOutput,
      cacheRead: model.costCacheRead ?? 0,
      cacheWrite: model.costCacheWrite ?? 0,
    },
    contextWindow: model.limitContext ?? 0,
    maxTokens: model.limitOutput ?? 0,
  }
}

function buildProviderConfig(
  provider: DaemonProviderInfo,
  models: DaemonModelInfo[],
  credentials: DaemonCredentialsResponse,
): ProviderConfig {
  const providerModels = models.filter((m) => m.providerId === provider.id)
  const staticModels = providerModels.map(transformModel)

  const config: ProviderConfig = {
    baseUrl: credentials.baseUrl,
    apiKey: credentials.apiKey,
    api: "openai-completions",
    models: staticModels,
    fetchDynamicModels: async () => staticModels,
  }

  if (credentials.workspaceId) {
    config.headers = { "x-workspace-id": credentials.workspaceId }
  }

  return config
}

export default async function extension(pi: ExtensionAPI): Promise<void> {
  const daemonUrl = getDaemonUrl()

  const [modelsData, credentials] = await Promise.all([
    fetchModels(daemonUrl),
    fetchCredentials(daemonUrl),
  ])

  if (!modelsData || !credentials) {
    pi.logger.warn(
      "Failed to fetch models or credentials from omcd, skipping provider injection",
    )
  } else if (modelsData.providers.length === 0) {
    pi.logger.warn(
      "No providers available from omcd, skipping provider injection",
    )
  } else {
    for (const provider of modelsData.providers) {
      const config = buildProviderConfig(
        provider,
        modelsData.models,
        credentials,
      )
      pi.registerProvider(provider.id, config)
    }
    pi.logger.info(
      `Injected ${modelsData.providers.length} providers with ${modelsData.models.length} models from omcd`,
    )
  }

  pi.on("message_end", (event: MessageEndEvent, ctx: ExtensionContext) => {
    const { message } = event

    if (message.role !== "assistant") return
    if (!message.usage) return

    const record: TokenUsageRecord = {
      sessionId: ctx.sessionManager.getSessionId(),
      messageId: message.responseId ?? generateMessageId(),
      agent: "omp",
      model: message.model ?? "unknown",
      inputTokens: message.usage.input ?? 0,
      outputTokens: message.usage.output ?? 0,
      reasoningTokens: 0,
      cacheReadTokens: message.usage.cacheRead ?? 0,
      cacheWriteTokens: message.usage.cacheWrite ?? 0,
      recordedAt: Date.now(),
    }

    sendToDaemon(daemonUrl, record, pi.logger)
  })
}
