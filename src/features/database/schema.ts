import { sqliteTable, text, integer, real, primaryKey } from "drizzle-orm/sqlite-core"

export const accounts = sqliteTable("accounts", {
  id: text("id").primaryKey(),
  email: text("email").notNull(),
  url: text("url").notNull(),
  accessToken: text("access_token").notNull(),
  refreshToken: text("refresh_token").notNull(),
  tokenExpiry: integer("token_expiry"),
})

export const accountState = sqliteTable("account_state", {
  id: integer("id").primaryKey(),
  activeAccountId: text("active_account_id"),
  activeWorkspaceId: text("active_workspace_id"),
})

export const providers = sqliteTable("providers", {
  id: text("id").notNull(),
  name: text("name").notNull(),
  api: text("api"),
  npm: text("npm"),
  doc: text("doc"),
  envVars: text("env_vars").notNull(),
  accountId: text("account_id").notNull(),
  lastFetchedAt: integer("last_fetched_at"),
  createdAt: integer("created_at").notNull(),
  updatedAt: integer("updated_at").notNull(),
}, (t) => ({
  pk: primaryKey({ columns: [t.id, t.accountId] }),
}))

export const modelRecords = sqliteTable("models", {
  id: text("id").notNull(),
  providerId: text("provider_id").notNull(),
  name: text("name").notNull(),
  family: text("family"),
  attachment: integer("attachment"),
  reasoning: integer("reasoning"),
  toolCall: integer("tool_call"),
  enable: integer("enable"),
  structuredOutput: integer("structured_output"),
  temperature: integer("temperature"),
  interleavedField: text("interleaved_field"),
  knowledge: text("knowledge"),
  releaseDate: text("release_date"),
  lastUpdated: text("last_updated"),
  openWeights: integer("open_weights"),
  modalitiesInput: text("modalities_input"),
  modalitiesOutput: text("modalities_output"),
  costInput: real("cost_input").default(0),
  costOutput: real("cost_output").default(0),
  costReasoning: real("cost_reasoning").default(0),
  costCacheRead: real("cost_cache_read").default(0),
  costCacheWrite: real("cost_cache_write").default(0),
  limitContext: integer("limit_context"),
  limitOutput: integer("limit_output"),
  accountId: text("account_id").notNull(),
  createdAt: integer("created_at").notNull(),
  updatedAt: integer("updated_at").notNull(),
}, (t) => ({
  pk: primaryKey({ columns: [t.id, t.providerId, t.accountId] }),
}))
