import { Command } from "commander"
import { login } from "./login"
import { logout } from "./logout"
import { switchWorkspace } from "./switch"
import { listAccounts } from "./list"

export function createAccountCommand(): Command {
  const account = new Command("account")
    .description("Manage workspace accounts")

  account
    .command("login <url>")
    .description("Log in via device code flow")
    .action(async (url: string) => {
      const exitCode = await login(url)
      process.exit(exitCode)
    })

  account
    .command("logout [email]")
    .description("Log out from an account")
    .action(async (email: string | undefined) => {
      const exitCode = await logout(email)
      process.exit(exitCode)
    })

  account
    .command("switch")
    .description("Switch active workspace")
    .action(async () => {
      const exitCode = await switchWorkspace()
      process.exit(exitCode)
    })

  account
    .command("list")
    .description("List logged-in accounts")
    .action(async () => {
      const exitCode = await listAccounts()
      process.exit(exitCode)
    })

  return account
}

export { login, logout, switchWorkspace, listAccounts }
