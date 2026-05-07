import { Cause, Duration, Effect, Option } from "effect"

import type {
  AccountID,
  AccountInfo,
  WorkspaceID,
} from "../../features/account"
import {
  AccountError,
  defaultLayer,
  PollResult,
  PollSuccess,
  Service as Account,
} from "../../features/account"
import {
  createSpinner,
  intro,
  logInfo,
  openBrowser,
  outro,
  selectWorkspaceEffect,
} from "./ui"
import { refreshAfterLogin } from "./refresh-after-login"

const selectWorkspaceAfterLogin = Effect.gen(function* () {
  const service = yield* Account

  const groups = yield* service.workspacesByAccount()
  if (groups.length === 0) {
    yield* outro("No workspaces found")
    return
  }

  const active = yield* service
    .active()
    .pipe(Effect.catchAll(() => Effect.succeed(Option.none())))

  const activeOpt = Option.map(active, (a: AccountInfo) => ({
    id: a.id as unknown as AccountID,
    active_workspace_id: a.activeWorkspaceId as WorkspaceID | null,
  }))

  const selected = yield* selectWorkspaceEffect(groups, activeOpt)

  if (Option.isSome(selected)) {
    const choice = selected.value
    yield* service.use(choice.accountID, Option.some(choice.workspaceID))
  }
})

export const loginEffect = (
  url: string,
): Effect.Effect<void, AccountError, never> =>
  Effect.gen(function* () {
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
        TimeoutException: () =>
          Effect.succeed({ _tag: "PollExpired" } as PollResult),
      }),
    )

    if (result._tag === "PollSuccess") {
      yield* spinner.stop("Logged in as " + (result as PollSuccess).email)
      yield* selectWorkspaceAfterLogin
    } else if (result._tag === "PollExpired") {
      yield* spinner.stop("Device code expired", 1)
    } else if (result._tag === "PollDenied") {
      yield* spinner.stop("Authorization denied", 1)
    } else if (result._tag === "PollError") {
      yield* spinner.stop(
        "Error: " + String((result as { cause: unknown }).cause),
        1,
      )
    }
  }) as unknown as Effect.Effect<void, AccountError, never>

export async function login(url: string): Promise<number> {
  const result = await Effect.runPromiseExit(
    loginEffect(url).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    await refreshAfterLogin()
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
