import * as p from "@clack/prompts"
import color from "picocolors"
import { PLUGIN_NAME } from "../shared"
import type { InstallArgs } from "./types"
import {
  addPluginToOpenCodeConfig,
  detectCurrentConfig,
  getOpenCodeVersion,
  isOpenCodeInstalled,
  writeOmoConfig,
} from "./config-manager"
import {
  createDefaultInstallConfig,
  detectedToInitialValues,
  formatConfigSummary,
  SYMBOLS,
} from "./install-validators"
import { getUnsupportedOpenCodeVersionMessage } from "./minimum-opencode-version"
import { hasAnyAccount } from "./account/check-account-exists"
import { login } from "./account/login"

export async function runTuiInstaller(
  args: InstallArgs,
  version: string,
): Promise<number> {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    console.error(
      "Error: Interactive installer requires a TTY. Use --non-interactive or set environment variables directly.",
    )
    return 1
  }

  const detected = detectCurrentConfig()
  const isUpdate = detected.isInstalled

  p.intro(
    color.bgMagenta(
      color.white(isUpdate ? " oMoMoMoMo... Update " : " oMoMoMoMo... "),
    ),
  )

  if (isUpdate) {
    const initial = detectedToInitialValues(detected)
    p.log.info(
      `Existing configuration detected: Claude=${initial.claude}, Gemini=${initial.gemini}`,
    )
  }

  const spinner = p.spinner()
  spinner.start("Checking OpenCode installation")

  const installed = await isOpenCodeInstalled()
  const openCodeVersion = await getOpenCodeVersion()
  if (!installed) {
    spinner.stop(`OpenCode binary not found ${color.yellow("[!]")}`)
    p.log.warn(
      "OpenCode binary not found. Plugin will be configured, but you'll need to install OpenCode to use it.",
    )
    p.note(
      "Visit https://opencode.ai/docs for installation instructions",
      "Installation Guide",
    )
  } else {
    spinner.stop(
      `OpenCode ${openCodeVersion ?? "installed"} ${color.green("[OK]")}`,
    )

    const unsupportedVersionMessage =
      getUnsupportedOpenCodeVersionMessage(openCodeVersion)
    if (unsupportedVersionMessage) {
      p.log.warn(unsupportedVersionMessage)
      p.outro(color.red("Installation blocked."))
      return 1
    }
  }

  const config = createDefaultInstallConfig()

  spinner.start(`Adding ${PLUGIN_NAME} to OpenCode config`)
  const pluginResult = await addPluginToOpenCodeConfig(version)
  if (!pluginResult.success) {
    spinner.stop(`Failed to add plugin: ${pluginResult.error}`)
    p.outro(color.red("Installation failed."))
    return 1
  }
  spinner.stop(`Plugin added to ${color.cyan(pluginResult.configPath)}`)

  spinner.start(`Writing ${PLUGIN_NAME} configuration`)
  const omoResult = writeOmoConfig(config)
  if (!omoResult.success) {
    spinner.stop(`Failed to write config: ${omoResult.error}`)
    p.outro(color.red("Installation failed."))
    return 1
  }
  spinner.stop(`Config written to ${color.cyan(omoResult.configPath)}`)

  if (
    !config.hasClaude &&
    !config.hasOpenAI &&
    !config.hasGemini &&
    !config.hasCopilot &&
    !config.hasOpencodeZen &&
    !config.hasVercelAiGateway
  ) {
    p.log.warn(
      "No model providers configured. Using opencode/big-pickle as fallback.",
    )
  }

  p.note(
    formatConfigSummary(config),
    isUpdate ? "Updated Configuration" : "Installation Complete",
  )

  p.log.success(
    color.bold(isUpdate ? "Configuration updated!" : "Installation complete!"),
  )
  p.log.message(`Run ${color.cyan("opencode")} to start!`)
  p.log.info(
    "Anonymous telemetry is enabled by default. Disable it with OMO_SEND_ANONYMOUS_TELEMETRY=0 or OMO_DISABLE_POSTHOG=1.",
  )
  p.log.info(
    "Docs: docs/legal/privacy-policy.md and docs/legal/terms-of-service.md",
  )

  p.note(
    `Include ${color.cyan("ultrawork")} (or ${color.cyan("ulw")}) in your prompt.\n` +
      `All features work like magic-parallel agents, background tasks,\n` +
      `deep exploration, and relentless execution until completion.`,
    "The Magic Word",
  )

  p.outro(color.green("oMoMoMoMo... Enjoy!"))

  const hasAccounts = await hasAnyAccount()
  if (!hasAccounts) {
    const shouldLogin = await p.confirm({
      message: "No accounts found. Would you like to log in now?",
      initialValue: false,
    })

    if (shouldLogin) {
      const serverUrl = await p.text({
        message: "Enter server URL:",
        validate: (value) => {
          if (!value.trim()) return "Server URL is required"
          try {
            new URL(value)
          } catch {
            return "Invalid URL"
          }
          return undefined
        },
      })

      if (typeof serverUrl === "string") {
        console.log()
        const exitCode = await login(serverUrl)
        if (exitCode === 0) {
          p.log.success("Successfully logged in!")
        } else {
          p.log.warn("Login was not completed successfully.")
          p.log.info(
            `You can log in later with: ${color.cyan("bunx oh-my-codes account login <server-url>")}`,
          )
        }
      }
    } else {
      p.log.info(
        `You can log in later with: ${color.cyan("bunx oh-my-codes account login <server-url>")}`,
      )
    }
  }

  if (config.hasClaude || config.hasGemini || config.hasCopilot) {
    const providers: string[] = []
    if (config.hasClaude)
      providers.push(`Anthropic ${color.gray("→ Claude Pro/Max")}`)
    if (config.hasGemini) providers.push(`Google ${color.gray("→ Gemini")}`)
    if (config.hasCopilot) providers.push(`GitHub ${color.gray("→ Copilot")}`)

    console.log()
    console.log(color.bold("Authenticate Your Providers"))
    console.log()
    console.log(`   Run ${color.cyan("opencode auth login")} and select:`)
    for (const provider of providers) {
      console.log(`   ${SYMBOLS.bullet} ${provider}`)
    }
    console.log()
  }

  return 0
}
