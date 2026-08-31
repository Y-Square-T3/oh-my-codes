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
  costCacheRead?: number
  costCacheWrite?: number
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
