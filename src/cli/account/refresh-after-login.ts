import { Effect } from "effect"
import * as p from "@clack/prompts"

import { defaultLayer, Service } from "../../features/model"

export interface RefreshResult {
  providers: number
  models: number
}

export async function refreshAfterLogin(): Promise<RefreshResult | null> {
  const spinner = p.spinner()
  spinner.start("Refreshing models from your workspace...")

  try {
    const result = await Effect.runPromiseExit(
      Effect.gen(function* () {
        const svc = yield* Service
        return yield* svc.refresh()
      }).pipe(Effect.provide(defaultLayer)),
    )

    if (result._tag === "Success") {
      spinner.stop(`Refreshed ${result.value.providers} providers, ${result.value.models} models from workspace`)
      return {
        providers: result.value.providers,
        models: result.value.models,
      }
    }

    spinner.stop("Model refresh skipped")
    return null
  } catch {
    spinner.stop("Model refresh skipped")
    return null
  }
}
