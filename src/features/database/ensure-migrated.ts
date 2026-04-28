import { join, dirname } from "node:path"
import { fileURLToPath } from "node:url"
import { existsSync, mkdirSync } from "node:fs"
import { Database as BunDatabase } from "bun:sqlite"
import { drizzle } from "drizzle-orm/bun-sqlite"
import { migrate } from "drizzle-orm/bun-sqlite/migrator"

function resolveDbPath(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  const xdgConfig = process.env.XDG_CONFIG_HOME ?? join(process.env.HOME ?? "", ".config")
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

const currentFile = fileURLToPath(import.meta.url)
const isInSource = currentFile.includes("/src/") || currentFile.includes("\\src\\")
const MIGRATIONS_DIR = isInSource
  ? join(dirname(currentFile), "migrations")
  : join(dirname(dirname(currentFile)), "migrations")

export async function ensureMigrated(): Promise<void> {
  const dbPath = resolveDbPath()

  const dbDir = dbPath.substring(0, dbPath.lastIndexOf("/"))
  if (!existsSync(dbDir)) {
    mkdirSync(dbDir, { recursive: true })
  }

  const sqlite = new BunDatabase(dbPath)
  const db = drizzle(sqlite)

  try {
    migrate(db, { migrationsFolder: MIGRATIONS_DIR })
  } catch {
    // non-fatal -- let downstream commands handle errors naturally
  } finally {
    sqlite.close()
  }
}
