import { dirname, join } from "node:path"
import { homedir } from "node:os"
import { fileURLToPath } from "node:url"
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import pc from "picocolors"
import initSqlJs from "sql.js"
import { drizzle } from "drizzle-orm/sql-js"
import { migrate } from "drizzle-orm/sql-js/migrator"

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
    const fileDir = dirname(fileURLToPath(import.meta.url))
    const siblingMigrations = join(fileDir, "migrations")
    if (existsSync(siblingMigrations)) {
      return siblingMigrations
    }
    return join(fileDir, "migrations")
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

  try {
    const SQL = await initSqlJs()
    let buffer: Uint8Array | undefined

    if (existsSync(dbPath)) {
      buffer = readFileSync(dbPath)
    }

    const sqlite = new SQL.Database(buffer)
    const db = drizzle(sqlite)

    migrate(db, { migrationsFolder: MIGRATIONS_DIR })
    writeFileSync(dbPath, sqlite.export())
    sqlite.close()
  } catch (err) {
    migrationError(err)
    process.exit(1)
  }
}
