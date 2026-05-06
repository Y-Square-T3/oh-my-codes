import { join, dirname } from "node:path"
import { mkdirSync, existsSync } from "node:fs"
import { fileURLToPath } from "node:url"
import { Database as BunDatabase } from "bun:sqlite"
import { drizzle, type BunSQLiteDatabase } from "drizzle-orm/bun-sqlite"
import { migrate } from "drizzle-orm/bun-sqlite/migrator"
import { Context, Effect, Layer } from "effect"
import * as schema from "./schema"

const currentFile = fileURLToPath(import.meta.url)
const isInSource = currentFile.includes("/src/") || currentFile.includes("\\src\\")
const MIGRATIONS_DIR = process.env.OH_MY_CODES_ROOT
  ? join(process.env.OH_MY_CODES_ROOT, "dist", "migrations")
  : isInSource
    ? join(dirname(currentFile), "migrations")
    : join(dirname(dirname(currentFile)), "migrations")

console.log(`dir: ${MIGRATIONS_DIR}`)

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
  const xdgConfig = process.env.XDG_CONFIG_HOME ?? join(process.env.HOME ?? "", ".config")
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

export { schema }

export interface DatabaseService {
  readonly db: BunSQLiteDatabase<typeof schema>
  readonly sqlite: BunDatabase
  readonly migrate: () => Effect.Effect<void, DatabaseQueryError>
  readonly close: () => Effect.Effect<void>
}

export const Database = Context.GenericTag<DatabaseService>("@account/Database")

const createDefaultLayer = (dbPath?: string) =>
  Layer.sync(Database, () => {
    const path = dbPath ?? resolveDbPath()
    const dir = path.substring(0, path.lastIndexOf("/"))

    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true })
    }

    const sqlite = new BunDatabase(path)
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
