import { Cause, Effect } from "effect"
import type { PluginContext } from "../types"
import * as Model from "../features/model"
import { log } from "../features/log/logger"

export type Result<T> = { ok: true; value: T } | { ok: false; error: string }

export async function showToast(ctx: PluginContext, message: string, variant: "success" | "error" | "warning"): Promise<void> {
  try {
    await ctx.client.tui.showToast({
      body: { message, variant },
    })
  } catch (err) {
    log("[omc] Failed to show toast", { error: err })
  }
}

export async function runRefreshModels(): Promise<string | null> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const modelSvc = yield* Model.Service
      const result = yield* modelSvc.refresh()
      return `Refreshed ${result.providers} providers, ${result.models} models`
    }).pipe(Effect.provide(Model.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return exit.value
  }
  const cause = Cause.pretty(exit.cause)
  log("[omc] Model refresh failed", { error: cause })
  return null
}
