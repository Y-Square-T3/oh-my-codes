import { existsSync, realpathSync } from "node:fs"
import { homedir } from "node:os"
import { join, resolve } from "node:path"

export interface OpenCodeConfigPaths {
  configDir: string
  configJson: string
  configJsonc: string
  omoConfig: string
}

function resolveConfigPath(pathValue: string): string {
  const resolvedPath = resolve(pathValue)
  if (!existsSync(resolvedPath)) return resolvedPath

  try {
    return realpathSync(resolvedPath)
  } catch {
    return resolvedPath
  }
}

function getCliConfigDir(): string {
  const envConfigDir = process.env.OPENCODE_CONFIG_DIR?.trim()
  if (envConfigDir) {
    return resolveConfigPath(envConfigDir)
  }

  const xdgConfig = process.env.XDG_CONFIG_HOME || join(homedir(), ".config")
  return resolveConfigPath(join(xdgConfig, "opencode"))
}

export function getOpenCodeConfigPaths(): OpenCodeConfigPaths {
  const configDir = getCliConfigDir()

  return {
    configDir,
    configJson: join(configDir, "opencode.json"),
    configJsonc: join(configDir, "opencode.jsonc"),
    omoConfig: join(configDir, "oh-my-codes.json"),
  }
}
