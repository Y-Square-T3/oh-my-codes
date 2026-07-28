import type { Plugin } from "@opencode-ai/plugin"
import type {
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
    client: "opencode",
    sessionId: info.sessionID,
    messageId: info.id ?? info.messageID ?? "",
    agent: info.agent ?? null,
    providerId: info.providerID ?? "unknown",
    modelId: info.modelID ?? "unknown",
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
    client: "opencode",
    sessionId: p.sessionID,
    messageId: p.assistantMessageID ?? "",
    agent: p.agent ?? null,
    providerId: p.providerID ?? "unknown",
    modelId: p.modelID ?? "unknown",
    inputTokens: p.tokens.input ?? 0,
    outputTokens: p.tokens.output ?? 0,
    reasoningTokens: p.tokens.reasoning ?? 0,
    cacheReadTokens: p.tokens.cache?.read ?? 0,
    cacheWriteTokens: p.tokens.cache?.write ?? 0,
    recordedAt: Date.now(),
  }
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
