import { Command } from "commander"
import { runTuiInstaller } from "./tui-installer"
import { run } from "./run"
import { getLocalVersion } from "./get-local-version"
import { doctor } from "./doctor"
import { refreshModelCapabilities } from "./refresh-model-capabilities"
import { createAccountCommand } from "./account"
import { createModelCommand } from "./model"
import { createMcpOAuthCommand } from "./mcp-oauth"
import { runTokenUsages } from "./token-usages"
import { ensureMigrated } from "../features/database/ensure-migrated"
import type { RunOptions } from "./run"
import type { GetLocalVersionOptions } from "./get-local-version/types"
import type { DoctorOptions } from "./doctor"
import packageJson from "../../package.json" with { type: "json" }

const VERSION = packageJson.version

const program = new Command()

program
  .name("oh-my-codes")
  .description(
    "The ultimate OpenCode plugin - multi-model orchestration, LSP tools, and more",
  )
  .version(VERSION, "-v, --version", "Show version number")
  .enablePositionalOptions()

program
  .command("install")
  .description("Install and configure oh-my-codes with interactive setup")
  .option(
    "--skip-login",
    "Skip the account login prompt",
  )
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
  .command("run <message>")
  .allowUnknownOption()
  .passThroughOptions()
  .description("Run opencode with todo/background task completion enforcement")
  .option(
    "-a, --agent <name>",
    "Agent to use (default: from CLI/env/config, fallback: Sisyphus)",
  )
  .option(
    "-m, --model <provider/model>",
    "Model override (e.g., anthropic/claude-sonnet-4)",
  )
  .option("-d, --directory <path>", "Working directory")
  .option(
    "-p, --port <port>",
    "Server port (attaches if port already in use)",
    parseInt,
  )
  .option("--attach <url>", "Attach to existing opencode server URL")
  .option("--on-complete <command>", "Shell command to run after completion")
  .option("--json", "Output structured JSON result to stdout")
  .option("--no-timestamp", "Disable timestamp prefix in run output")
  .option("--verbose", "Show full event stream (default: messages/tools only)")
  .option(
    "--session-id <id>",
    "Resume existing session instead of creating new one",
  )
  .addHelpText(
    "after",
    `
Examples:
  $ bunx oh-my-codes run "Fix the bug in index.ts"
  $ bunx oh-my-codes run --agent Sisyphus "Implement feature X"
  $ bunx oh-my-codes run --port 4321 "Fix the bug"
  $ bunx oh-my-codes run --attach http://127.0.0.1:4321 "Fix the bug"
  $ bunx oh-my-codes run --json "Fix the bug" | jq .sessionId
  $ bunx oh-my-codes run --on-complete "notify-send Done" "Fix the bug"
  $ bunx oh-my-codes run --session-id ses_abc123 "Continue the work"
  $ bunx oh-my-codes run --model anthropic/claude-sonnet-4 "Fix the bug"
  $ bunx oh-my-codes run --agent Sisyphus --model openai/gpt-5.4 "Implement feature X"

Agent resolution order:
  1) --agent flag
  2) OPENCODE_DEFAULT_AGENT
  3) oh-my-codes.json "default_run_agent"
  4) Sisyphus (fallback)

Available core agents:
  Sisyphus, Hephaestus, Prometheus, Atlas

Unlike 'opencode run', this command waits until:
  - All todos are completed or cancelled
  - All child sessions (background tasks) are idle
`,
  )
  .action(async (message: string, options) => {
    if (options.port && options.attach) {
      console.error("Error: --port and --attach are mutually exclusive")
      process.exit(1)
    }
    const runOptions: RunOptions = {
      message,
      agent: options.agent,
      model: options.model,
      directory: options.directory,
      port: options.port,
      attach: options.attach,
      onComplete: options.onComplete,
      json: options.json ?? false,
      timestamp: options.timestamp ?? true,
      verbose: options.verbose ?? false,
      sessionId: options.sessionId,
    }
    const exitCode = await run(runOptions)
    process.exit(exitCode)
  })

program
  .command("get-local-version")
  .description("Show current installed version and check for updates")
  .option("-d, --directory <path>", "Working directory to check config from")
  .option("--json", "Output in JSON format for scripting")
  .addHelpText(
    "after",
    `
Examples:
  $ bunx oh-my-codes get-local-version
  $ bunx oh-my-codes get-local-version --json
  $ bunx oh-my-codes get-local-version --directory /path/to/project

This command shows:
  - Current installed version
  - Latest available version on npm
  - Whether you're up to date
  - Special modes (local dev, pinned version)
`,
  )
  .action(async (options) => {
    const versionOptions: GetLocalVersionOptions = {
      directory: options.directory,
      json: options.json ?? false,
    }
    const exitCode = await getLocalVersion(versionOptions)
    process.exit(exitCode)
  })

program
  .command("doctor")
  .description("Check oh-my-codes installation health and diagnose issues")
  .option("--status", "Show compact system dashboard")
  .option("--verbose", "Show detailed diagnostic information")
  .option("--json", "Output results in JSON format")
  .addHelpText(
    "after",
    `
Examples:
  $ bunx oh-my-codes doctor            # Show problems only
  $ bunx oh-my-codes doctor --status   # Compact dashboard
  $ bunx oh-my-codes doctor --verbose  # Deep diagnostics
  $ bunx oh-my-codes doctor --json     # JSON output
`,
  )
  .action(async (options) => {
    const mode = options.status
      ? "status"
      : options.verbose
        ? "verbose"
        : "default"
    const doctorOptions: DoctorOptions = {
      mode,
      json: options.json ?? false,
    }
    const exitCode = await doctor(doctorOptions)
    process.exit(exitCode)
  })

program
  .command("refresh-model-capabilities")
  .description(
    "Refresh the cached models.dev-based model capabilities snapshot",
  )
  .option(
    "-d, --directory <path>",
    "Working directory to read oh-my-codes config from",
  )
  .option("--source-url <url>", "Override the models.dev source URL")
  .option("--json", "Output refresh summary as JSON")
  .action(async (options) => {
    const exitCode = await refreshModelCapabilities({
      directory: options.directory,
      sourceUrl: options.sourceUrl,
      json: options.json ?? false,
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

program.addCommand(createMcpOAuthCommand())
program.addCommand(createAccountCommand())
program.addCommand(createModelCommand())

export async function runCli(): Promise<void> {
  await ensureMigrated()
  program.parse()
}
