import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"
import { existsSync, mkdirSync } from "node:fs"
import pc from "picocolors"
import { Database as BunDatabase } from "bun:sqlite"
import { drizzle } from "drizzle-orm/bun-sqlite"
import { migrate } from "drizzle-orm/bun-sqlite/migrator"

const SUPPORT_URL = "https://github.com/Y-Square-T3/oh-my-codes/issues"

function resolveDbPath(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  const xdgConfig =
    process.env.XDG_CONFIG_HOME ?? join(process.env.HOME ?? "", ".config")
  const configDir = envConfigDir ?? join(xdgConfig, "opencode")

  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true })
  }

  return join(configDir, "oh-my-codes.db")
}

const currentFile = fileURLToPath(import.meta.url)
const MIGRATIONS_DIR = process.env.OH_MY_CODES_ROOT
  ? join(process.env.OH_MY_CODES_ROOT, "dist", "migrations")
  : join(dirname(currentFile), "migrations")

console.error(pc.red(`dir: ${MIGRATIONS_DIR}`))

function migrationError(err: unknown): void {
  const message = err instanceof Error ? err.message : String(err)
  console.error(pc.red("error: database migration failed"))
  console.error(pc.dim(message))
  console.error()
  console.error(
    pc.dim("please report this issue to our team:"),
    pc.cyan(SUPPORT_URL),
  )
}

export async function ensureMigrated(): Promise<void> {
  const dbPath = resolveDbPath()

  const dbDir = dbPath.substring(0, dbPath.lastIndexOf("/"))
  if (!existsSync(dbDir)) {
    mkdirSync(dbDir, { recursive: true })
  }

  console.log(`add db path: ${dbPath}`)

  const sqlite = new BunDatabase(dbPath)
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
