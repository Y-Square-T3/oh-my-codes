import { type Provider, Model } from "./type"
import { type providers, type modelRecords } from "./schema"
import { type AccountID } from "../account/schema"

export interface TransformedProvider {
  id: string
  name: string
  api: string | null
  npm: string | null
  doc: string | null
  envVars: string
  accountId: AccountID
  lastFetchedAt: number
  createdAt: number
  updatedAt: number
}

export interface TransformedModel {
  id: string
  providerId: string
  name: string
  family: string | null
  attachment: boolean | null
  reasoning: boolean | null
  toolCall: boolean | null
  enable: boolean | null
  structuredOutput: boolean | null
  temperature: boolean | null
  interleavedField: string | null
  knowledge: string | null
  releaseDate: string | null
  lastUpdated: string | null
  openWeights: boolean | null
  modalitiesInput: string | null
  modalitiesOutput: string | null
  costInput: number
  costOutput: number
  costReasoning: number
  costCacheRead: number
  costCacheWrite: number
  limitContext: number | null
  limitOutput: number | null
  accountId: AccountID
  createdAt: number
  updatedAt: number
}

const now = () => Date.now()

export function transformProvider(
  providerId: string,
  provider: Provider,
  accountId: AccountID,
): TransformedProvider {
  return {
    id: providerId,
    name: provider.name,
    api: provider.api ?? null,
    npm: provider.npm ?? null,
    doc: provider.doc ?? null,
    envVars: JSON.stringify(provider.env),
    accountId,
    lastFetchedAt: now(),
    createdAt: now(),
    updatedAt: now(),
  }
}

export function transformModel(
  providerId: string,
  modelId: string,
  model: Model,
  accountId: AccountID,
): TransformedModel {
  const cost = model.cost
  const limit = model.limit

  let interleavedField: string | null = null
  if (model.interleaved !== undefined && model.interleaved !== true) {
    interleavedField = model.interleaved.field
  }

  let modalitiesInput: string | null = null
  let modalitiesOutput: string | null = null
  if (model.modalities) {
    modalitiesInput = JSON.stringify(model.modalities.input)
    modalitiesOutput = JSON.stringify(model.modalities.output)
  }

  return {
    id: modelId,
    providerId,
    name: model.name,
    family: model.family ?? null,
    attachment: model.attachment ?? null,
    reasoning: model.reasoning ?? null,
    toolCall: model.tool_call ?? null,
    enable: true,
    structuredOutput: model.structured_output ?? null,
    temperature: model.temperature ?? null,
    interleavedField,
    knowledge: model.knowledge ?? null,
    releaseDate: model.release_date ?? null,
    lastUpdated: model.last_updated ?? null,
    openWeights: model.open_weights ?? null,
    modalitiesInput,
    modalitiesOutput,
    costInput: cost?.input ?? 0,
    costOutput: cost?.output ?? 0,
    costReasoning: cost?.reasoning ?? 0,
    costCacheRead: cost?.cache_read ?? 0,
    costCacheWrite: cost?.cache_write ?? 0,
    limitContext: limit?.context ?? null,
    limitOutput: limit?.output ?? null,
    accountId,
    createdAt: now(),
    updatedAt: now(),
  }
}
