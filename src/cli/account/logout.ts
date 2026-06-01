import { Effect, Option, Cause } from "effect"

import { Service as Account, defaultLayer } from "../../features/account"
import { formatAccountLabel, intro, outro, println, select } from "./ui"
import type { AccountInfo, AccountError } from "../../features/account"

export const logoutEffect = (email?: string): Effect.Effect<void, AccountError, never> =>
  Effect.gen(function* () {
    const service = yield* Account
    const accounts = yield* service.list()

    if (accounts.length === 0) {
      yield* println("Not logged in")
      return
    }

    if (email) {
      const match = accounts.find((a: AccountInfo) => a.email === email)
      if (!match) {
        yield* println("Account not found: " + email)
        return
      }
      yield* service.remove(match.id)
      yield* outro("Logged out from " + email)
      return
    }

    const active = yield* service.active().pipe(Effect.catchAll(() => Effect.succeed(Option.none())))

    const activeID = Option.map(active, (a: AccountInfo) => a.id)

    yield* intro("Log out")

    const opts = accounts.map((a: AccountInfo) => {
      const isActive = Option.isSome(activeID) && (activeID.value as string) === a.id
      return {
        value: a,
        label: formatAccountLabel(a, isActive),
      }
    })

    const selected = yield* select(opts, "Select account to log out")
    if (Option.isNone(selected)) return

    const acc = selected.value as AccountInfo
    yield* service.remove(acc.id)
    yield* outro("Logged out from " + acc.email)
  }) as unknown as Effect.Effect<void, AccountError, never>

export async function logout(email?: string): Promise<number> {
  const result = await Effect.runPromiseExit(logoutEffect(email).pipe(Effect.provide(defaultLayer)))

  if (result._tag === "Success") {
    return 0
  }

  const causeStr = Cause.pretty(result.cause)
  console.error(`Error: ${causeStr}`)
  return 1
}
