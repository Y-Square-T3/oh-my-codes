import { dirname, join } from "node:path"
import { homedir } from "node:os"
import { fileURLToPath } from "node:url"
import { existsSync, mkdirSync } from "node:fs"
import pc from "picocolors"
import SqliteDatabase from "better-sqlite3"
import { drizzle } from "drizzle-orm/better-sqlite3"
import { migrate } from "drizzle-orm/better-sqlite3/migrator"

const SUPPORT_URL = "https://github.com/Y-Square-T3/oh-my-codes/issues"

function resolveDbPath(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  const xdgConfig = process.env.XDG_CONFIG_HOME ?? join(homedir(), ".config")
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

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

function migrationError(err: unknown): void {
  const message = err instanceof Error ? err.message : String(err)
  console.error(pc.red("error: database migration failed"))
  console.error(pc.dim(message))
  console.error()
  console.error(pc.dim("please report this issue to our team:"), pc.cyan(SUPPORT_URL))
}

export async function ensureMigrated(): Promise<void> {
  const dbPath = resolveDbPath()

  const dbDir = dirname(dbPath)
  if (!existsSync(dbDir)) {
    mkdirSync(dbDir, { recursive: true })
  }

  const sqlite = new SqliteDatabase(dbPath)
  const db = drizzle(sqlite)

  try {
    migrate(db, { migrationsFolder: MIGRATIONS_DIR })
  } catch (err) {
    migrationError(err)
    process.exit(1)
  } finally {
    sqlite.close()
  }
}
