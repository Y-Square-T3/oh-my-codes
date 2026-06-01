import { dirname, join } from "node:path"
import { homedir } from "node:os"
import { existsSync, mkdirSync } from "node:fs"
import { fileURLToPath } from "node:url"
import SqliteDatabase from "better-sqlite3"
import { type BetterSQLite3Database, drizzle } from "drizzle-orm/better-sqlite3"
import { migrate } from "drizzle-orm/better-sqlite3/migrator"
import { Context, Effect, Layer } from "effect"
import * as schema from "./schema"

function getMigrationsDir(): string {
  if (process.env.OH_MY_CODES_ROOT) {
    return join(process.env.OH_MY_CODES_ROOT, "dist", "migrations")
  }
  if (typeof import.meta?.url !== "undefined") {
    return join(dirname(dirname(fileURLToPath(import.meta.url))), "migrations")
  }
  return join(dirname(dirname(__filename)), "migrations")
}

const MIGRATIONS_DIR = getMigrationsDir()

export class DatabaseQueryError extends Error {
  readonly _tag = "DatabaseQueryError"

  constructor(
    message: string,
    readonly cause?: unknown,
  ) {
    super(message)
    this.name = "DatabaseQueryError"
  }
}

export function resolveDbPath(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  const xdgConfig = process.env.XDG_CONFIG_HOME ?? join(homedir(), ".config")
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

export { schema }

export interface DatabaseService {
  readonly db: BetterSQLite3Database<typeof schema>
  readonly sqlite: SqliteDatabase.Database
  readonly migrate: () => Effect.Effect<void, DatabaseQueryError>
  readonly close: () => Effect.Effect<void>
}

export const Database = Context.GenericTag<DatabaseService>("@account/Database")

const createDefaultLayer = (dbPath?: string) =>
  Layer.sync(Database, () => {
    const path = dbPath ?? resolveDbPath()
    const dir = dirname(path)

    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true })
    }

    const sqlite = new SqliteDatabase(path)
    const db = drizzle(sqlite, { schema })

    return {
      db,
      sqlite,
      migrate: () =>
        Effect.try({
          try: () => migrate(db, { migrationsFolder: MIGRATIONS_DIR }),
          catch: (cause) => {
            console.error(cause)
            return new DatabaseQueryError("Database migration failed", cause)
          },
        }),
      close: () => Effect.sync(() => sqlite.close()),
    }
  })

export const defaultLayer = createDefaultLayer()

export const makeLayer = (path: string) => createDefaultLayer(path)
