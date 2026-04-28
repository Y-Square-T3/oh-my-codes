import type { Migration } from "../database"

export const migration001: Migration = {
  id: 1,
  name: "create_accounts_and_account_state",
  up: `
    CREATE TABLE IF NOT EXISTS accounts (
      id TEXT PRIMARY KEY,
      email TEXT NOT NULL,
      url TEXT NOT NULL,
      access_token TEXT NOT NULL,
      refresh_token TEXT NOT NULL,
      token_expiry INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS account_state (
      id INTEGER PRIMARY KEY,
      active_account_id TEXT,
      active_workspace_id TEXT
    );
  `,
}
