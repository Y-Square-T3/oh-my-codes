import { Command } from "commander"
import { runTuiInstaller } from "./tui-installer"
import { createAccountCommand } from "./account"
import { createModelCommand } from "./model"
import { runTokenUsages } from "./token-usages"
import { ensureMigrated } from "../features/database/ensure-migrated"
import packageJson from "../../package.json" with { type: "json" }

const VERSION = packageJson.version

const program = new Command()

program
  .name("oh-my-codes")
  .description("The ultimate OpenCode plugin - multi-model orchestration, LSP tools, and more")
  .version(VERSION, "-v, --version", "Show version number")
  .enablePositionalOptions()

program
  .command("install")
  .description("Install and configure oh-my-codes with interactive setup")
  .option("--skip-login", "Skip the account login prompt")
  .addHelpText(
    "after",
    `
Examples:
  $ bunx oh-my-codes install
  $ bunx oh-my-codes install --skip-login

Model providers are auto-detected from your existing OpenCode authentication.
You can configure providers later in ~/.config/opencode/oh-my-codes.jsonc.
`,
  )
  .action(async (options) => {
    const exitCode = await runTuiInstaller(VERSION, {
      skipLogin: options.skipLogin,
    })
    process.exit(exitCode)
  })

program
  .command("token-usages")
  .alias("tu")
  .description("View and push locally cached token usage data")
  .argument("[action]", "Action: status (default) or push")
  .option("--json", "Output in JSON format")
  .option("--database <path>", "Path to database file")
  .addHelpText(
    "after",
    `
Examples:
  $ bunx oh-my-codes token-usages           # Show local status
  $ bunx oh-my-codes token-usages status    # Show unpushed count
  $ bunx oh-my-codes token-usages push      # Push all cached usages
  $ bunx oh-my-codes tu push --json         # Push with JSON output
`,
  )
  .action(async (action, options) => {
    const exitCode = await runTokenUsages({
      action: action as "status" | "push" | undefined,
      json: options.json ?? false,
      dbPath: options.database,
    })
    process.exit(exitCode)
  })

program
  .command("version")
  .description("Show version information")
  .action(() => {
    console.log(`oh-my-codes v${VERSION}`)
  })

program.addCommand(createAccountCommand())
program.addCommand(createModelCommand())

export async function runCli(): Promise<void> {
  await ensureMigrated()
  program.parse()
}
