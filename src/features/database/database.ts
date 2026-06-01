import { dirname, join } from "node:path"
import { homedir } from "node:os"
import { existsSync, mkdirSync } from "node:fs"
import { DatabaseSync } from "node:sqlite"
import { fileURLToPath } from "node:url"
import { type NodeSQLiteDatabase, drizzle } from "drizzle-orm/node-sqlite"
import { migrate } from "drizzle-orm/node-sqlite/migrator"
import { Context, Effect, Layer } from "effect"
import * as schema from "./schema"

function getMigrationsDir(): string {
  if (process.env.OH_MY_CODES_ROOT) {
    return join(process.env.OH_MY_CODES_ROOT, "dist", "migrations")
  }
  if (typeof import.meta?.url !== "undefined") {
    return join(dirname(fileURLToPath(import.meta.url)), "migrations")
  }
  return join(dirname(__filename), "migrations")
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
  readonly db: NodeSQLiteDatabase<typeof schema>
  readonly sqlite: DatabaseSync
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

    const sqlite = new DatabaseSync(path, {
      enableForeignKeyConstraints: true,
    })
    const db = drizzle({ client: sqlite, schema })

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
