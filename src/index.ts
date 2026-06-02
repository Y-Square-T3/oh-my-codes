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
          template: `You are helping the user login to their OMC (oh-my-codes) account.

$ARGUMENTS may contain the server URL. Follow these steps:

1. If $ARGUMENTS is empty or does not look like a valid URL, ask the user for their OMC server URL (e.g. https://server.omc.ai).
2. Once you have the URL, call the omc-login tool with it.
3. Based on the response:
   - If it starts with MISSING_URL: ask the user for their server URL and retry.
   - If it starts with LOGIN_FAILED or AUTH_FAILED: inform the user of the error.
   - If it starts with LOGIN_SUCCESS and says workspace was auto-selected: confirm success, the user is ready.
   - If it starts with LOGIN_SUCCESS and lists workspaces: present the workspace options and ask the user which one to use.
4. If the user selects a workspace, call the omc-switch tool with their selection (number or name).
5. Confirm the final state to the user.`,
        }
      }

      if (!cmd["omc-switch"]) {
        cmd["omc-switch"] = {
          description: "Switch OMC workspace",
          template: `You are helping the user switch their active OMC workspace.

$ARGUMENTS may contain a workspace number or name. Follow these steps:

1. If $ARGUMENTS is empty, call the omc-switch tool without arguments to list all available workspaces.
2. If $ARGUMENTS has a value, call the omc-switch tool with it as the id argument.
3. Based on the response:
   - If it starts with NO_WORKSPACES: inform the user they need to login first with /omc-login.
   - If it starts with WORKSPACE_LIST: present the workspace options to the user and ask which one to switch to.
   - If it starts with INVALID_SELECTION: show the error and list again, ask user to pick a valid option.
   - If it starts with SWITCH_FAILED: inform the user of the error.
   - If it starts with SWITCH_SUCCESS: confirm the switch was successful.
4. If the user makes a selection from the list, call omc-switch with their selection (number).
5. Confirm the final state to the user.`,
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
