import { Command } from "commander"

import { listModels, refreshModels, clearModels } from "./list"

export function createModelCommand(): Command {
  const model = new Command("model")
    .description("Manage models from account API")

  model
    .command("list")
    .description("List models from the account's model API")
    .option("--provider <id>", "Filter by provider ID")
    .option("--json", "Output as JSON")
    .action(async (opts) => {
      const exitCode = await listModels(opts)
      process.exit(exitCode)
    })

  model
    .command("refresh")
    .description("Fetch fresh models from the account API")
    .option("--json", "Output as JSON")
    .action(async (opts) => {
      const exitCode = await refreshModels(opts)
      process.exit(exitCode)
    })

  model
    .command("clear")
    .description("Clear models for the active account")
    .option("--provider <id>", "Clear only a specific provider")
    .option("--json", "Output as JSON")
    .action(async (opts) => {
      const exitCode = await clearModels(opts)
      process.exit(exitCode)
    })

  return model
}
