import { sqliteTable, text, integer, real, primaryKey } from "drizzle-orm/sqlite-core"
import { type AccountID } from "../account/schema"

export const providers = sqliteTable("providers", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  api: text("api"),
  npm: text("npm"),
  doc: text("doc"),
  envVars: text("env_vars").notNull(),
  accountId: text("account_id"),
  lastFetchedAt: integer("last_fetched_at"),
  createdAt: integer("created_at").notNull(),
  updatedAt: integer("updated_at").notNull(),
})

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
  accountId: text("account_id"),
  createdAt: integer("created_at").notNull(),
  updatedAt: integer("updated_at").notNull(),
}, (t) => ({
  pk: primaryKey({ columns: [t.id, t.providerId] }),
}))

export type ProviderRow = typeof providers.$inferSelect
export type ModelRow = typeof modelRecords.$inferSelect
