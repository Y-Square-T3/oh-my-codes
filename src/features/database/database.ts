import { basename, dirname, join } from "node:path"
import { homedir } from "node:os"
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { fileURLToPath } from "node:url"
import initSqlJs, { type Database as SqlJsDatabase } from "sql.js"
import { type SQLJsDatabase, drizzle } from "drizzle-orm/sql-js"
import { migrate } from "drizzle-orm/sql-js/migrator"
import { Context, Effect, Layer } from "effect"
import * as schema from "./schema"

function getMigrationsDir(): string {
  if (process.env.OH_MY_CODES_ROOT) {
    return join(process.env.OH_MY_CODES_ROOT, "dist", "migrations")
  }
  if (typeof import.meta?.url !== "undefined") {
    const fileDir = dirname(fileURLToPath(import.meta.url))
    if (basename(fileDir) === "dist") {
      return join(fileDir, "migrations")
    }
    return join(dirname(fileDir), "migrations")
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
  readonly db: SQLJsDatabase<typeof schema>
  readonly sqlite: SqlJsDatabase
  readonly flush: () => Effect.Effect<void, DatabaseQueryError>
  readonly migrate: () => Effect.Effect<void, DatabaseQueryError>
  readonly close: () => Effect.Effect<void>
}

export const Database = Context.GenericTag<DatabaseService>("@account/Database")

async function createSqlJsInstance(dbPath: string) {
  const SQL = await initSqlJs()
  let buffer: Uint8Array | undefined

  if (existsSync(dbPath)) {
    buffer = readFileSync(dbPath)
  }

  return {
    sqlite: new SQL.Database(buffer),
    SQL,
  }
}

function createDefaultLayer(dbPath: string): Layer.Layer<DatabaseService> {
  return Layer.effect(
    Database,
    Effect.promise(async () => {
      const dir = dirname(dbPath)
      if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true })
      }

      const { sqlite } = await createSqlJsInstance(dbPath)
      const db = drizzle(sqlite, { schema })

      const flushEffect = Effect.try({
        try: () => {
          writeFileSync(dbPath, sqlite.export())
        },
        catch: (cause) => new DatabaseQueryError("Failed to flush database", cause),
      })

      return {
        db,
        sqlite,
        flush: () => flushEffect,
        migrate: () =>
          Effect.try({
            try: () => migrate(db, { migrationsFolder: MIGRATIONS_DIR }),
            catch: (cause) => new DatabaseQueryError("Database migration failed", cause),
          }).pipe(Effect.flatMap(() => flushEffect)),
        close: () =>
          Effect.sync(() => {
            writeFileSync(dbPath, sqlite.export())
            sqlite.close()
          }),
      }
    }),
  )
}

export const defaultLayer = createDefaultLayer(resolveDbPath())

export const makeLayer = (path: string) => createDefaultLayer(path)
