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

export interface DaemonProviderInfo {
  id: string
  name: string
  api?: string
  npm?: string
  env: string[]
  modelCount: number
}

export interface DaemonModelInfo {
  id: string
  providerId: string
  name: string
  family?: string
  reasoning?: boolean
  toolCall?: boolean
  attachment?: boolean
  temperature?: boolean
  openWeights?: boolean
  modalitiesInput: string[]
  modalitiesOutput: string[]
  costInput: number
  costOutput: number
  limitContext?: number
  limitOutput?: number
  releaseDate?: string
}

export interface DaemonModelsListResponse {
  providers: DaemonProviderInfo[]
  models: DaemonModelInfo[]
  accountEmail?: string
  accountUrl?: string
}

export interface DaemonCredentialsResponse {
  apiKey: string
  baseUrl: string
  workspaceId?: string
}
