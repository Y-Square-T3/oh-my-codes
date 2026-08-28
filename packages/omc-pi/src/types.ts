export interface AgentMessage {
  role: string
  model?: string
  id?: string
  usage?: {
    input?: number
    output?: number
    cacheRead?: number
    cacheWrite?: number
  }
}

export interface MessageEndEvent {
  type: "message_end"
  message: AgentMessage
}

export interface SessionManager {
  getSessionId(): string
}

export interface ExtensionContext {
  sessionManager: SessionManager
}

export interface Logger {
  info(...args: unknown[]): void
  warn(...args: unknown[]): void
  error(...args: unknown[]): void
  debug(...args: unknown[]): void
}

export interface ExtensionAPI {
  on(
    event: "message_end",
    handler: (event: MessageEndEvent, ctx: ExtensionContext) => void,
  ): void
  logger: Logger
}
