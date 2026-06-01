import { getOpenCodeConfigPaths } from "./opencode-config-dir"
import type { OpenCodeConfigPaths } from "./opencode-config-dir"

export interface ConfigContext {
  paths: OpenCodeConfigPaths
}

let configContext: ConfigContext | null = null

export function initConfigContext(): void {
  configContext = { paths: getOpenCodeConfigPaths() }
}

export function getConfigContext(): ConfigContext {
  if (!configContext) {
    configContext = { paths: getOpenCodeConfigPaths() }
  }
  return configContext
}

export function resetConfigContext(): void {
  configContext = null
}

export function getConfigDir(): string {
  return getConfigContext().paths.configDir
}

export function getConfigJson(): string {
  return getConfigContext().paths.configJson
}

export function getConfigJsonc(): string {
  return getConfigContext().paths.configJsonc
}
