import { Duration, Effect, Option, Cause } from "effect"

import { Service as Account, defaultLayer } from "../../features/account"
import { createSpinner, intro, logInfo, openBrowser, outro, selectWorkspaceEffect } from "./ui"
import { AccountError, PollResult, PollSuccess } from "../../features/account"
import type { AccountInfo, AccountID, WorkspaceID } from "../../features/account"

const selectWorkspaceAfterLogin = Effect.gen(function* () {
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
    active_workspace_id: a.active_workspace_id as WorkspaceID | null,
  }))

  const selected = yield* selectWorkspaceEffect(groups, activeOpt)

  if (Option.isSome(selected)) {
    const choice = selected.value
    yield* service.use(choice.accountID, Option.some(choice.workspaceID))
    yield* service.sync()
  }
})

export const loginEffect = (url: string): Effect.Effect<void, AccountError, never> =>
  (Effect.gen(function* () {
    const service = yield* Account

    yield* intro("Log in")

    const login = yield* service.login(url)

    yield* logInfo(`Go to: ${login.url}`)
    yield* logInfo(`Enter code: ${login.user}`)

    yield* openBrowser(login.url)

    const spinner = createSpinner()
    yield* spinner.start("Waiting for authorization...")

    const pollLoop = (wait: number): Effect.Effect<PollResult, AccountError> =>
      Effect.gen(function* () {
        yield* Effect.sleep(Duration.seconds(wait))
        const result = yield* service.poll(login)
        if (result._tag === "PollPending") return yield* pollLoop(wait)
        if (result._tag === "PollSlow") return yield* pollLoop(wait + 5)
        return result
      })

    const result = yield* pollLoop(login.interval).pipe(
      Effect.timeout(Duration.seconds(login.expiry)),
      Effect.catchTags({
        TimeoutException: () => Effect.succeed({ _tag: "PollExpired" } as PollResult),
      }),
    )

    if (result._tag === "PollSuccess") {
      yield* Effect.sync(() => spinner.stop("Logged in as " + (result as PollSuccess).email))
      yield* selectWorkspaceAfterLogin
    } else if (result._tag === "PollExpired") {
      yield* Effect.sync(() => spinner.stop("Device code expired", 1))
    } else if (result._tag === "PollDenied") {
      yield* Effect.sync(() => spinner.stop("Authorization denied", 1))
    } else if (result._tag === "PollError") {
      yield* Effect.sync(() => spinner.stop("Error: " + String((result as { cause: unknown }).cause), 1))
    }
  }) as unknown as Effect.Effect<void, AccountError, never>)

export async function login(url: string): Promise<number> {
  const result = await Effect.runPromiseExit(
    loginEffect(url).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return 0
  }

  const error = Cause.failureOption(result.cause)
  if (Option.isSome(error) && error.value instanceof Error) {
    console.error(`Error: ${error.value.message}`)
  } else {
    const causeStr = Cause.pretty(result.cause)
    console.error(`Error: ${causeStr}`)
  }
  return 1
}
