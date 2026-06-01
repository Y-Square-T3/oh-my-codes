import * as p from "@clack/prompts"
import color from "picocolors"
import { hasAnyAccount } from "./account/check-account-exists"
import { login } from "./account/login"

export interface TuiInstallerOptions {
  skipLogin?: boolean
}

export async function runTuiInstaller(version: string, options?: TuiInstallerOptions): Promise<number> {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    console.error(
      "Error: Interactive installer requires a TTY. Set environment variables directly in ~/.config/opencode/oh-my-codes.jsonc for scripted installation.",
    )
    return 1
  }

  if (!options?.skipLogin) {
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
            if (!value?.trim()) return "Server URL is required"
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
            p.log.success("Model capabilities refreshed!")
          } else {
            p.log.warn("Login was not completed successfully.")
            p.log.info(`You can log in later with: ${color.cyan("bunx oh-my-codes account login <server-url>")}`)
          }
        }
      } else {
        p.log.info(`You can log in later with: ${color.cyan("bunx oh-my-codes account login <server-url>")}`)
      }
    }
  }

  return 0
}
