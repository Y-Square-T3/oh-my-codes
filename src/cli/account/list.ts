import { Effect, Option, Cause } from "effect"

import { Service as Account, defaultLayer } from "../../features/account"
import { isActiveWorkspaceChoice, formatOrgLine, println } from "./ui"
import type { AccountInfo, AccountID, WorkspaceID } from "../../features/account"

export const listEffect = Effect.gen(function* () {
  const service = yield* Account

  const groups = yield* service.workspacesByAccount()
  if (groups.length === 0) {
    yield* println("No accounts found")
    return
  }

  if (!groups.some((group) => group.workspaces.length > 0)) {
    yield* println("No workspaces found")
    return
  }

  const active = yield* service.active().pipe(
    Effect.catchAll(() => Effect.succeed(Option.none())),
  )

  const activeOpt = Option.map(active, (a: AccountInfo) => ({
    id: a.id as unknown as AccountID,
    active_workspace_id: a.activeWorkspaceId as WorkspaceID | null,
  }))

  for (const group of groups) {
    for (const workspace of group.workspaces) {
      const isActive = activeOpt
        ? isActiveWorkspaceChoice(activeOpt, {
            accountID: group.account.id,
            workspaceID: workspace.id,
          })
        : false
      yield* println(formatOrgLine(group.account, workspace, isActive))
    }
  }
})

export async function listAccounts(): Promise<number> {
  const result = await Effect.runPromiseExit(
    listEffect.pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(`Error: ${causeStr}`)
  return 1
}
