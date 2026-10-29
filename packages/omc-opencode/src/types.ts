export interface TokenUsageRecord {
  agent: string
  model: string
  sessionId: string
  messageId: string
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

export interface V2StepTokens {
  input?: number
  output?: number
  reasoning?: number
  cache?: { read?: number; write?: number }
}

export interface V2StepStartedData {
  sessionID?: string
  assistantMessageID?: string
  agent?: string
  model?: { id?: string; providerID?: string; variant?: string }
}

export interface V2StepEndedData {
  sessionID?: string
  assistantMessageID?: string
  finish?: string
  rawFinish?: string
  cost?: number
  tokens?: V2StepTokens
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
