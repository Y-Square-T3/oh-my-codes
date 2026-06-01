import { createRequire } from "node:module"
import { dirname } from "node:path"
import type { Hooks, Plugin, PluginModule } from "@opencode-ai/plugin"
import { applyAccountProviderConfig } from "./plugin-handlers/account-provider-config-handler"
import { recordTokenUsage } from "./hooks/audit-token-tracker"
import { log } from "./features/log/logger"

const require_local = createRequire(import.meta.url)
const packageRoot = dirname(require_local.resolve("oh-my-codes/package.json"))
process.env.OH_MY_CODES_ROOT = packageRoot

const serverPlugin: Plugin = async (input): Promise<Hooks> => {
  log("[oh-my-codes] ENTRY - plugin loading", { directory: input.directory })

  return {
    config: async (config: Record<string, unknown>) => {
      await applyAccountProviderConfig({ config })
    },

    event: async (eventInput) => {
      await recordTokenUsage(eventInput)
    },
  }
}

const pluginModule: PluginModule = {
  id: "oh-my-codes",
  server: serverPlugin,
}

export default pluginModule
