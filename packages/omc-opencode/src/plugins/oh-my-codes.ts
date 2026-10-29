import type { Plugin as V1Plugin } from "@opencode-ai/plugin"
import type { Plugin as V2 } from "@opencode/plugin"
import type {
  DaemonCredentialsResponse,
  DaemonModelInfo,
  DaemonModelsListResponse,
  DaemonProviderInfo,
  TokenUsageRecord,
  V1MessageUpdatedEvent,
  V2StepEndedData,
  V2StepStartedData,
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

// Emits one usage record per `session.step.ended` event. In OpenCode v2 each
// assistant message gets its own step with its own incremental token counts
// (verified live against `session.usage.updated` session totals), so one
// message is exactly one record, matching V1 `message.updated` semantics and
// the daemon's upsert-by-(agent, messageID) identity. `session.step.started`
// supplies agent/model, which `step.ended` does not carry.
export function createV2UsageAggregator() {
  const meta = new Map<
    string,
    { sessionId: string; agent: string; model: string }
  >()
  const seen = new Set<string>()

  return {
    onStarted(data: V2StepStartedData | undefined): void {
      const messageID = data?.assistantMessageID
      if (!messageID || !data) return
      if (!meta.has(messageID)) {
        meta.set(messageID, {
          sessionId: data.sessionID ?? "",
          agent: data.agent ?? "",
          model: data.model?.id ?? "unknown",
        })
      }
    },

    onEnded(data: V2StepEndedData | undefined): TokenUsageRecord | null {
      const messageID = data?.assistantMessageID
      if (!messageID || !data || seen.has(messageID)) return null
      seen.add(messageID)
      const base = meta.get(messageID) ?? {
        sessionId: data.sessionID ?? "",
        agent: "",
        model: "unknown",
      }
      meta.delete(messageID)
      if (isCompactionAgent(base.agent)) return null

      return {
        sessionId: data.sessionID || base.sessionId,
        messageId: messageID,
        agent: base.agent,
        model: base.model,
        inputTokens: data.tokens?.input ?? 0,
        outputTokens: data.tokens?.output ?? 0,
        reasoningTokens: data.tokens?.reasoning ?? 0,
        cacheReadTokens: data.tokens?.cache?.read ?? 0,
        cacheWriteTokens: data.tokens?.cache?.write ?? 0,
        recordedAt: Date.now(),
      }
    },
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

function groupModelsByProvider(
  models: DaemonModelInfo[],
): Map<string, DaemonModelInfo[]> {
  const modelsByProvider = new Map<string, DaemonModelInfo[]>()
  for (const model of models) {
    const existing = modelsByProvider.get(model.providerId) ?? []
    existing.push(model)
    modelsByProvider.set(model.providerId, existing)
  }
  return modelsByProvider
}

function buildProviderConfig(
  providers: DaemonProviderInfo[],
  models: DaemonModelInfo[],
  credentials: DaemonCredentialsResponse,
): Record<string, unknown> {
  const modelsByProvider = groupModelsByProvider(models)

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

function providerPackage(npm?: string): string {
  const base = npm && npm.length > 0 ? npm : "@ai-sdk/openai-compatible"
  if (base.startsWith("aisdk:")) return base
  return `aisdk:${base}`
}

function buildV2Model(
  providerID: string,
  model: DaemonModelInfo,
): Record<string, unknown> {
  const capabilities: Record<string, unknown> = {
    tools: model.toolCall ?? false,
    input: model.modalitiesInput,
    output: model.modalitiesOutput,
  }
  if (model.temperature) capabilities.temperature = true

  const released = model.releaseDate ? Date.parse(model.releaseDate) : 0

  const info: Record<string, unknown> = {
    id: model.id,
    modelID: model.id,
    providerID,
    name: model.name,
    capabilities,
    variants: [],
    time: { released: Number.isNaN(released) ? 0 : released },
    cost: [
      {
        input: model.costInput,
        output: model.costOutput,
        cache: { read: 0, write: 0 },
      },
    ],
    status: "active",
    enabled: true,
    limit: {
      context: model.limitContext ?? 200_000,
      output: model.limitOutput ?? 32_000,
    },
  }
  if (model.family) info.family = model.family

  return info
}

export function buildV2ProviderSources(
  modelsData: DaemonModelsListResponse,
  credentials: DaemonCredentialsResponse,
): { info: Record<string, unknown>; models: Record<string, unknown>[] }[] {
  const modelsByProvider = groupModelsByProvider(modelsData.models)

  return modelsData.providers.map((provider) => {
    const info: Record<string, unknown> = {
      id: provider.id,
      name: provider.name,
      activation: "enabled",
      package: providerPackage(provider.npm),
      settings: {
        baseURL: credentials.baseUrl,
        apiKey: credentials.apiKey,
      },
    }
    if (credentials.workspaceId) {
      info.headers = { "x-workspace-id": credentials.workspaceId }
    }

    const models = (modelsByProvider.get(provider.id) ?? []).map((model) =>
      buildV2Model(provider.id, model),
    )

    return { info, models }
  })
}

export async function setupV2(ctx: V2.Context): Promise<V2.Cleanup | void> {
  const daemonUrl = getDaemonUrl()
  const log = async (msg: string) => {
    console.log(`[oh-my-codes] ${msg}`)
  }

  const [modelsData, credentials] = await Promise.all([
    fetchModels(daemonUrl),
    fetchCredentials(daemonUrl),
  ])

  if (!modelsData || !credentials || modelsData.providers.length === 0) {
    await log(
      "Failed to fetch models or credentials from omcd, skipping provider injection",
    )
  } else {
    const sources = buildV2ProviderSources(modelsData, credentials)
    await ctx.provider.transform((editor) => {
      for (const source of sources) {
        // User-configured providers win, mirroring the V1 merge semantics.
        if (editor.get(source.info.id as string)) continue
        editor.add(source as unknown as Parameters<typeof editor.add>[0])
      }
    })
    await log(
      `Injected ${modelsData.providers.length} providers with ${modelsData.models.length} models from omcd`,
    )
  }

  const controller = new AbortController()
  const aggregator = createV2UsageAggregator()

  void (async () => {
    try {
      for await (const event of ctx.event.subscribe({
        types: ["session.step.started", "session.step.ended"],
        signal: controller.signal,
      })) {
        if (event.type === "session.step.started") {
          aggregator.onStarted(event.data)
        } else if (event.type === "session.step.ended") {
          const record = aggregator.onEnded(event.data)
          if (record) {
            await sendToDaemon(daemonUrl, record, log)
          }
        }
      }
    } catch (e) {
      if (!controller.signal.aborted) {
        console.warn(`[oh-my-codes] event stream error: ${e}`)
      }
    }
  })()

  return () => {
    controller.abort()
  }
}

export const OhMyCodesPlugin: V1Plugin = async ({ client }) => {
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

      if (e.type === "message.updated") {
        const record = extractV1(e as unknown as V1MessageUpdatedEvent)
        if (record) {
          await sendToDaemon(daemonUrl, record, log)
        }
      }
    },
  }
}

export const OhMyCodesPluginDual = {
  id: "oh-my-codes",
  setup: setupV2,
  server: OhMyCodesPlugin,
}
