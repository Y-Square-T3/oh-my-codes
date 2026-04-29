import { Effect, Cause } from "effect"
import pc from "picocolors"

import { Service, defaultLayer } from "../../features/model"

export async function listModels(opts: { provider?: string; json?: boolean }): Promise<number> {
  const result = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const svc = yield* Service
      const listResult = yield* svc.list(opts.provider)

      if (opts.json) {
        console.log(JSON.stringify(listResult, null, 2))
        return
      }

      if (listResult.providers.length === 0) {
        console.log("No models found. Run `model refresh` to fetch models from your account.")
        return
      }

      if (listResult.accountEmail) {
        console.log(pc.bold(`Models for: ${listResult.accountEmail}`))
        if (listResult.accountUrl) {
          console.log(pc.dim(listResult.accountUrl))
        }
        console.log()
      }

      for (const provider of listResult.providers) {
        const providerModels = listResult.models.filter((m) => m.providerId === provider.id)

        console.log(pc.bold(`${provider.name} (${provider.id})`) + pc.dim(` — ${providerModels.length} models`))
        console.log(pc.dim("─".repeat(60)))

        if (providerModels.length === 0) {
          console.log(pc.dim("  (no models)"))
        } else {
          const header = `  ${pc.dim("Model")}  ${pc.dim("Family")}  ${pc.dim("Reason")}  ${pc.dim("Tool")}  ${pc.dim("Context")}`
          console.log(pc.cyan(header))

          for (const m of providerModels) {
            const context = m.limitContext ? `${(m.limitContext / 1000).toFixed(0)}k` : "-"
            console.log(
              `  ${m.name.padEnd(12)} ${(m.family ?? "-").padEnd(8)} ${m.reasoning ? pc.green("yes") : pc.dim("no").padEnd(6)} ${m.toolCall ? pc.green("yes") : pc.dim("no").padEnd(6)} ${pc.dim(context)}`,
            )
          }
        }

        console.log()
      }
    }).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(pc.red(`Error: ${causeStr}`))
  return 1
}

export async function refreshModels(opts: { json?: boolean }): Promise<number> {
  const result = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const svc = yield* Service
      const refreshResult = yield* svc.refresh()

      if (opts.json) {
        console.log(JSON.stringify(refreshResult, null, 2))
        return
      }

      console.log(pc.green(`✓`) + ` Refreshed ${refreshResult.providers} providers, ${refreshResult.models} models`)
    }).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(pc.red(`Error: ${causeStr}`))
  return 1
}

export async function clearModels(opts: { provider?: string; json?: boolean }): Promise<number> {
  const result = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const svc = yield* Service
      const clearResult = yield* svc.clear(opts.provider)

      if (opts.json) {
        console.log(JSON.stringify(clearResult, null, 2))
        return
      }

      if (clearResult.modelsDeleted === 0) {
        console.log(pc.yellow("No models to clear."))
        return
      }

      console.log(
        pc.green(`✓`) + ` Cleared ${clearResult.modelsDeleted} models${clearResult.providersDeleted > 0 ? `, ${clearResult.providersDeleted} providers` : ""}`,
      )
    }).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(pc.red(`Error: ${causeStr}`))
  return 1
}
