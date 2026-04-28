import { join } from "node:path"
import { mkdirSync, existsSync } from "node:fs"
import { Database as BunDatabase, type Database as BunDatabaseType } from "bun:sqlite"
import { Context, Effect, Layer, Option } from "effect"

type SqlParam = string | number | bigint | boolean | null | Uint8Array

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

export interface Migration {
  readonly id: number
  readonly name: string
  readonly up: string
}

export interface DatabaseService {
  readonly db: BunDatabaseType
  readonly run: (sql: string, params?: SqlParam[]) => Effect.Effect<void, DatabaseQueryError>
  readonly runAll: <T>(sql: string, params?: SqlParam[]) => Effect.Effect<T[], DatabaseQueryError>
  readonly runOne: (sql: string, params?: SqlParam[]) => Effect.Effect<Option.Option<unknown>, DatabaseQueryError>
  readonly migrate: (migrations: readonly Migration[]) => Effect.Effect<void, DatabaseQueryError>
  readonly close: () => Effect.Effect<void>
}

export const Database = Context.GenericTag<DatabaseService>("@account/Database")

function getDbPath(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  const xdgConfig = process.env.XDG_CONFIG_HOME ?? join(
    process.env.HOME ?? "",
    ".config",
  )
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

const migrationsTableSql = `
  CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applied_at INTEGER NOT NULL DEFAULT (unixepoch())
  )
`

const createDefaultLayer = (dbPath?: string) =>
  Layer.sync(Database, () => {
    const path = dbPath ?? getDbPath()
    const dir = path.substring(0, path.lastIndexOf("/"))

    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true })
    }

    const db = new BunDatabase(path)
    db.run(migrationsTableSql)

    return {
      db,
      run: (sql, params) =>
        Effect.try({
          try: () => {
            db.run(sql, params ?? [])
          },
          catch: (cause) => new DatabaseQueryError(`Query failed: ${sql}`, cause),
        }),
      runAll: <T>(sql: string, params?: SqlParam[]) =>
        Effect.try({
          try: () => db.query(sql).all(...(params ?? [])) as T[],
          catch: (cause) => new DatabaseQueryError(`Query failed: ${sql}`, cause),
        }),
      runOne: (sql, params) =>
        Effect.succeed(Option.fromNullable(db.query(sql).get(...(params ?? [])))),
      migrate: (migrations) =>
        Effect.gen(function* () {
          const applied: { id: number }[] = yield* Effect.sync(() =>
            (db.query("SELECT id FROM _migrations ORDER BY id").all() as { id: number }[]),
          )
          const appliedIds = new Set(applied.map((r) => r.id))

          for (const migration of migrations) {
            if (!appliedIds.has(migration.id)) {
              yield* Effect.try({
                try: () => db.run(migration.up),
                catch: (cause) =>
                  new DatabaseQueryError(`Migration ${migration.name} failed`, cause),
              })
              yield* Effect.try({
                try: () =>
                  db.run(
                    "INSERT INTO _migrations (id, name) VALUES (?, ?)",
                    [migration.id, migration.name],
                  ),
                catch: (cause) =>
                  new DatabaseQueryError(`Failed to record migration ${migration.name}`, cause),
              })
            }
          }
        }),
      close: () => Effect.sync(() => db.close()),
    }
  })

export const defaultLayer = createDefaultLayer()

export const makeLayer = (path: string) => createDefaultLayer(path)
