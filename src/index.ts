import { createRequire } from "node:module"
import { dirname } from "node:path"
import type { Hooks, Plugin, PluginModule } from "@opencode-ai/plugin"
import { applyAccountProviderConfig } from "./plugin-handlers/account-provider-config-handler"
import { recordTokenUsage } from "./hooks/audit-token-tracker"
import { createLoginTool } from "./plugin-tools/create-login-tool"
import { createSwitchTool } from "./plugin-tools/create-switch-tool"
import { log } from "./features/log/logger"

const require_local = createRequire(import.meta.url)
const packageRoot = dirname(require_local.resolve("oh-my-codes/package.json"))
process.env.OH_MY_CODES_ROOT = packageRoot

const serverPlugin: Plugin = async (input): Promise<Hooks> => {
  log("[oh-my-codes] ENTRY - plugin loading", { directory: input.directory })

  return {
    tool: {
      "omc-login": createLoginTool(input),
      "omc-switch": createSwitchTool(input),
    },

    config: async (config: Record<string, unknown>) => {
      await applyAccountProviderConfig({ config })

      if (!config.command) {
        config.command = {}
      }

      const cmd = config.command as Record<string, unknown>

      if (!cmd["omc-login"]) {
        cmd["omc-login"] = {
          description: "Login to an OMC account",
          template: "Use the omc-login tool to authenticate with your OMC account server.",
        }
      }

      if (!cmd["omc-switch"]) {
        cmd["omc-switch"] = {
          description: "Switch OMC workspace",
          template: "Use the omc-switch tool to switch between your OMC workspaces.",
        }
      }
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
