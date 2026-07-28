export interface TokenUsageRecord {
  client: string
  sessionId: string
  messageId: string
  agent?: string | null
  providerId: string
  modelId: string
  inputTokens: number
  outputTokens: number
  reasoningTokens: number
  cacheReadTokens: number
  cacheWriteTokens: number
  recordedAt?: number
}

export interface V1MessageUpdatedEvent {
  type: "message.updated"
  properties?: {
    info?: {
      role?: string
      sessionID?: string
      id?: string
      messageID?: string
      agent?: string
      providerID?: string
      modelID?: string
      finish?: boolean
      tokens?: {
        total?: number
        input?: number
        output?: number
        reasoning?: number
        cache?: { read?: number; write?: number }
      }
    }
  }
}

export interface V2StepEndedEvent {
  type: "session.next.step.ended"
  properties?: {
    sessionID?: string
    assistantMessageID?: string
    agent?: string
    providerID?: string
    modelID?: string
    finish?: string
    cost?: number
    tokens?: {
      input?: number
      output?: number
      reasoning?: number
      cache?: { read?: number; write?: number }
    }
  }
}
