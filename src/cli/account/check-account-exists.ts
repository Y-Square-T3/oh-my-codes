import { Effect, Cause } from "effect"

import { defaultLayer, Service as Account } from "../../features/account"

export async function hasAnyAccount(): Promise<boolean> {
  const result = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const service = yield* Account
      const accounts = yield* service.list()
      return accounts.length > 0
    }).pipe(Effect.provide(defaultLayer)),
  )

  if (result._tag === "Success") {
    return result.value
  }

  return false
}
