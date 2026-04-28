import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core"

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
