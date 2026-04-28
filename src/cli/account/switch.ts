import { Effect, Option, Cause } from "effect"

import { Service as Account, defaultLayer } from "../../features/account"
import { intro, outro, selectWorkspaceEffect } from "./ui"
import type { AccountInfo, AccountID, WorkspaceID } from "../../features/account"

export const switchEffect = Effect.gen(function* () {
  yield* intro("Switch workspace")

  const service = yield* Account
  const groups = yield* service.workspacesByAccount()

  if (groups.length === 0) {
    yield* outro("No workspaces found")
    return
  }

  const active = yield* service.active().pipe(
    Effect.catchAll(() => Effect.succeed(Option.none())),
  )

  const activeOpt = Option.map(active, (a: AccountInfo) => ({
    id: a.id as unknown as AccountID,
    active_workspace_id: a.activeWorkspaceId as WorkspaceID | null,
  }))

  const selected = yield* selectWorkspaceEffect(groups, activeOpt)

  if (Option.isSome(selected)) {
    const choice = selected.value
    yield* service.use(choice.accountID, Option.some(choice.workspaceID))
    yield* outro("Switched to " + choice.label)
  }
})

export async function switchWorkspace(): Promise<number> {
  const result = await Effect.runPromiseExit(
    switchEffect.pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(`Error: ${causeStr}`)
  return 1
}
