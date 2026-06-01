import * as Schema from "effect/Schema"

export const AuditRecordID = Schema.String.pipe(Schema.brand("AuditRecordID"))
export type AuditRecordID = Schema.Schema.Type<typeof AuditRecordID>

export class TokenUsageRecord extends Schema.Class<TokenUsageRecord>("TokenUsageRecord")({
  id: AuditRecordID,
  recordedAt: Schema.Number,
  sessionID: Schema.String,
  messageID: Schema.String,
  agent: Schema.NullOr(Schema.String),
  providerID: Schema.String,
  modelID: Schema.String,
  inputTokens: Schema.Number,
  outputTokens: Schema.Number,
  reasoningTokens: Schema.Number,
  cacheReadTokens: Schema.Number,
  cacheWriteTokens: Schema.Number,
  pushed: Schema.Boolean,
  createdAt: Schema.Number,
}) {}

export class AuditRepoError extends Schema.TaggedClass<AuditRepoError>()("AuditRepoError", {
  message: Schema.String,
  cause: Schema.optional(Schema.Defect),
}) {}

export class AuditServiceError extends Schema.TaggedClass<AuditServiceError>()("AuditServiceError", {
  message: Schema.String,
  cause: Schema.optional(Schema.Defect),
}) {}

export type AuditError = AuditRepoError | AuditServiceError

export interface PushResult {
  pushedCount: number
  failedCount: number
  ids: string[]
}
