import { readFileSync, writeFileSync } from "node:fs"
import type { ConfigMergeResult } from "./types"
import { PLUGIN_NAME } from "../../utils/plugin-identity"
import { backupConfigFile } from "./backup-config"
import { getConfigDir } from "./config-context"
import { ensureConfigDirectoryExists } from "./ensure-config-directory-exists"
import { formatErrorWithSuggestion } from "./format-error-with-suggestion"
import { detectConfigFormat } from "./opencode-config-format"
import { type OpenCodeConfig, parseOpenCodeConfigFileWithError } from "./parse-opencode-config-file"
import { checkVersionCompatibility, extractVersionFromPluginEntry } from "./version-compatibility"

const PLUGIN_ENTRY = "oh-my-codes@latest"

async function handleNoConfig(path: string): Promise<ConfigMergeResult> {
  try {
    const config: OpenCodeConfig = { plugin: [PLUGIN_ENTRY] }
    writeFileSync(path, JSON.stringify(config, null, 2) + "\n")
    return { success: true, configPath: path }
  } catch (err) {
    return {
      success: false,
      configPath: path,
      error: formatErrorWithSuggestion(err, "create opencode config"),
    }
  }
}

async function handleExistingConfig(path: string, format: "json" | "jsonc"): Promise<ConfigMergeResult> {
  const parseResult = await parseOpenCodeConfigFileWithError(path)
  if (!parseResult.config) {
    return {
      success: false,
      configPath: path,
      error: parseResult.error ?? "Failed to parse config file",
    }
  }

  const config = parseResult.config
  const plugins = config.plugin ?? []

  const canonicalEntries = plugins.filter((plugin) => plugin === PLUGIN_NAME || plugin.startsWith(`${PLUGIN_NAME}@`))
  const otherPlugins = plugins.filter((plugin) => !(plugin === PLUGIN_NAME || plugin.startsWith(`${PLUGIN_NAME}@`)))

  const existingEntry = canonicalEntries[0]
  if (existingEntry) {
    const installedVersion = extractVersionFromPluginEntry(existingEntry)
    const compatibility = checkVersionCompatibility(installedVersion, PLUGIN_ENTRY)

    if (!compatibility.canUpgrade) {
      return {
        success: false,
        configPath: path,
        error: compatibility.reason ?? "Version compatibility check failed",
      }
    }

    const backupResult = backupConfigFile(path)
    if (!backupResult.success) {
      return {
        success: false,
        configPath: path,
        error: `Failed to create backup: ${backupResult.error}`,
      }
    }
  }

  const normalizedPlugins = [...otherPlugins]
  normalizedPlugins.push(PLUGIN_ENTRY)
  config.plugin = normalizedPlugins

  return writeConfig(path, format, config)
}

function writeConfig(path: string, format: "json" | "jsonc", config: OpenCodeConfig): ConfigMergeResult {
  try {
    if (format === "jsonc") {
      const content = readFileSync(path, "utf-8")
      const pluginArrayRegex = /((?:"plugin"|plugin)\s*:\s*)\[([\s\S]*?)\]/
      const match = content.match(pluginArrayRegex)

      if (match) {
        const formattedPlugins = config.plugin!.map((p) => `"${p}"`).join(",\n    ")
        const newContent = content.replace(pluginArrayRegex, `$1[\n    ${formattedPlugins}\n  ]`)
        writeFileSync(path, newContent)
      } else {
        const pluginStr = JSON.stringify(config.plugin)
        const newContent = content.replace(/(\{)/, `$1\n  "plugin": ${pluginStr},`)
        writeFileSync(path, newContent)
      }
    } else {
      writeFileSync(path, JSON.stringify(config, null, 2) + "\n")
    }

    return { success: true, configPath: path }
  } catch (err) {
    return {
      success: false,
      configPath: path,
      error: formatErrorWithSuggestion(err, "write opencode config"),
    }
  }
}

export async function addPluginToOpenCodeConfig(): Promise<ConfigMergeResult> {
  try {
    ensureConfigDirectoryExists()
  } catch (err) {
    return {
      success: false,
      configPath: getConfigDir(),
      error: formatErrorWithSuggestion(err, "create config directory"),
    }
  }

  const { format, path } = detectConfigFormat()

  try {
    if (format === "none") {
      return handleNoConfig(path)
    }

    return handleExistingConfig(path, format)
  } catch (err) {
    return {
      success: false,
      configPath: path,
      error: formatErrorWithSuggestion(err, "update opencode config"),
    }
  }
}
