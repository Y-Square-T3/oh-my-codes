import type {
  ExtensionAPI,
  ExtensionContext,
  MessageEndEvent,
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

export default function extension(pi: ExtensionAPI): void {
  pi.on("message_end", (event: MessageEndEvent, ctx: ExtensionContext) => {
    const { message } = event

    if (message.role !== "assistant") return
    if (!message.usage) return

    const record: TokenUsageRecord = {
      sessionId: ctx.sessionManager.getSessionId(),
      messageId: message.id ?? generateMessageId(),
      agent: "omp",
      model: message.model ?? "unknown",
      inputTokens: message.usage.input ?? 0,
      outputTokens: message.usage.output ?? 0,
      reasoningTokens: 0,
      cacheReadTokens: message.usage.cacheRead ?? 0,
      cacheWriteTokens: message.usage.cacheWrite ?? 0,
      recordedAt: Date.now(),
    }

    const daemonUrl = getDaemonUrl()
    sendToDaemon(daemonUrl, record, pi.logger)
  })
}
