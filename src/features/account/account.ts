import { Clock, Duration, Effect, Layer, Match, Option, Schema as S, Context } from "effect"

import * as AccountRepo from "./repo"
import { normalizeServerUrl } from "./url"
import {
  AccountError,
  AccountID,
  AccessToken,
  AccountInfo as AccountInfoSchema,
  AccountServiceError,
  DeviceAuthResponse,
  DeviceToken,
  DeviceTokenError,
  DeviceTokenSuccess,
  DeviceCode,
  UserCode,
  Login,
  PollDenied,
  PollError,
  PollExpired,
  PollPending,
  PollResult,
  PollSlow,
  PollSuccess,
  RefreshToken,
  TokenRefreshResponse,
  User,
  Workspace,
  WorkspaceID,
} from "./schema"

export type AccountWorkspace = {
  account: AccountInfoSchema
  workspaces: readonly Workspace[]
}

const CLIENT_ID = "oh-my-codes"
const EAGER_REFRESH_THRESHOLD_MS = 5 * 60 * 1000

const isTokenFresh = (tokenExpiry: number | null, now: number): boolean =>
  tokenExpiry != null && tokenExpiry > now + EAGER_REFRESH_THRESHOLD_MS

const serviceError = (message: string, cause?: unknown) =>
  new AccountServiceError({ message, cause })

interface FetchOptions {
  method: string
  headers: Record<string, string>
  body?: string
}

const fetchJson = (url: string, options: FetchOptions) =>
  Effect.tryPromise({
    try: async () => {
      const response = await fetch(url, {
        method: options.method,
        headers: options.headers,
        body: options.body,
      })
      if (!response.ok) {
        throw serviceError(`HTTP ${response.status} from ${url}`)
      }
      return (await response.json()) as unknown
    },
    catch: (cause) => serviceError("HTTP request failed", cause),
  })

const decodeSchema = <A, I>(schema: S.Schema<A, I, never>, errorMsg: string) =>
  (json: unknown) =>
    Effect.try({
      try: () => S.decodeUnknownSync(schema)(json),
      catch: (cause) => serviceError(errorMsg, cause),
    })

export interface AccountServiceInterface {
  readonly active: () => Effect.Effect<Option.Option<AccountInfoSchema>, AccountError>
  readonly list: () => Effect.Effect<AccountInfoSchema[], AccountError>
  readonly workspacesByAccount: () => Effect.Effect<AccountWorkspace[], AccountError>
  readonly remove: (accountID: AccountID) => Effect.Effect<void, AccountError>
  readonly use: (accountID: AccountID, workspaceID: Option.Option<WorkspaceID>) => Effect.Effect<void, AccountError>
  readonly workspaces: (accountID: AccountID) => Effect.Effect<readonly Workspace[], AccountError>
  readonly token: (accountID: AccountID) => Effect.Effect<Option.Option<AccessToken>, AccountError>
  readonly login: (url: string) => Effect.Effect<Login, AccountError>
  readonly poll: (input: Login) => Effect.Effect<PollResult, AccountError>
  readonly refreshToken: (accountID: AccountID) => Effect.Effect<AccessToken, AccountError>
}

export const Service = Context.GenericTag<AccountServiceInterface>("@account/Account")

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const repo = yield* AccountRepo.Service

    const fetchUser = (serverUrl: string, accessToken: string) =>
      fetchJson(`${serverUrl}/api/me`, {
        method: "GET",
        headers: {
          Accept: "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
      }).pipe(Effect.flatMap(decodeSchema(User, "Failed to decode user response")))

    const fetchWorkspacesList = (serverUrl: string, accessToken: string) =>
      fetchJson(`${serverUrl}/api/workspaces`, {
        method: "GET",
        headers: {
          Accept: "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
      }).pipe(
        Effect.flatMap((json) =>
          Effect.try({
            try: () => {
              const arr = json as Record<string, unknown>[]
              return arr.map((item) => S.decodeUnknownSync(Workspace)(item))
            },
            catch: (cause) => serviceError("Failed to decode workspaces response", cause),
          }),
        ),
      )

    const refreshTokenInternal = (row: AccountRepo.AccountRow) =>
      Effect.gen(function* () {
        const response = yield* fetchJson(`${row.url}/auth/device/token`, {
          method: "POST",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            grant_type: "refresh_token",
            refresh_token: row.refresh_token,
            client_id: CLIENT_ID,
          }),
        })

        const parsed = yield* decodeSchema(TokenRefreshResponse, "Failed to decode token refresh response")(response)

        const now = yield* Clock.currentTimeMillis
        const expiry = now + parsed.expiresIn * 1000

        yield* repo.persistToken({
          accountID: row.id,
          accessToken: parsed.accessToken as AccessToken,
          refreshToken: parsed.refreshToken as RefreshToken,
          expiry: Option.some(expiry),
        })

        return parsed.accessToken as AccessToken
      })

    const resolveToken = (row: AccountRepo.AccountRow) =>
      Effect.gen(function* () {
        const now = yield* Clock.currentTimeMillis
        if (isTokenFresh(row.token_expiry, now)) {
          return row.access_token
        }
        return yield* refreshTokenInternal(row)
      })

    const resolveAccess = (accountID: AccountID) =>
      Effect.gen(function* () {
        const maybeAccount = yield* repo.getRow(accountID)
        if (Option.isNone(maybeAccount)) return Option.none()

        const account = maybeAccount.value
        const accessToken = yield* resolveToken(account)
        return Option.some({ account, accessToken })
      })

    const active = () =>
      repo.active().pipe(
        Effect.catchTag("AccountRepoError", (e) =>
          Effect.fail(serviceError("Failed to get active account", e.cause)),
        ),
      )

    const list = () =>
      repo.list().pipe(
        Effect.catchTag("AccountRepoError", (e) =>
          Effect.fail(serviceError("Failed to list accounts", e.cause)),
        ),
      )

    const workspacesByAccount = (): Effect.Effect<AccountWorkspace[], AccountError, never> =>
      Effect.gen(function* () {
        const accounts = yield* list()
        const results: AccountWorkspace[] = []

        for (const account of accounts) {
          const ws = yield* workspaces(account.id).pipe(
            Effect.catchAll(() => Effect.succeed([] as readonly Workspace[])),
          )
          results.push({ account, workspaces: ws })
        }

        return results
      }) as unknown as Effect.Effect<AccountWorkspace[], AccountError, never>

    const workspaces = (accountID: AccountID) =>
      Effect.gen(function* () {
        const resolved = yield* resolveAccess(accountID)
        if (Option.isNone(resolved)) return [] as readonly Workspace[]

        return yield* fetchWorkspacesList(resolved.value.account.url, resolved.value.accessToken)
      }) as unknown as Effect.Effect<readonly Workspace[], AccountError, never>

    const token = (accountID: AccountID): Effect.Effect<Option.Option<AccessToken>, AccountError, never> =>
      resolveAccess(accountID).pipe(
        Effect.map(Option.map((r) => r.accessToken)),
      ) as unknown as Effect.Effect<Option.Option<AccessToken>, AccountError, never>

    const refreshToken = (accountID: AccountID) =>
      Effect.gen(function* () {
        const maybeAccount = yield* repo.getRow(accountID)
        if (Option.isNone(maybeAccount)) {
          return yield* Effect.fail(serviceError("Account not found"))
        }
        return yield* refreshTokenInternal(maybeAccount.value)
      })

    const login = (url: string) =>
      Effect.gen(function* () {
        const normalizedServer = normalizeServerUrl(url)

        const response = yield* fetchJson(`${normalizedServer}/auth/device/code`, {
          method: "POST",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ client_id: CLIENT_ID }),
        })

        const parsed = yield* decodeSchema(DeviceAuthResponse, "Failed to decode device code response")(response)

        return new Login({
          code: parsed.deviceCode as unknown as DeviceCode,
          user: parsed.userCode as unknown as UserCode,
          url: `${normalizedServer}${parsed.verificationUriComplete}`,
          server: normalizedServer,
          expiry: parsed.expiresIn,
          interval: parsed.interval,
        })
      })

    const poll = (input: Login) =>
      Effect.gen(function* () {
        const response = yield* fetchJson(`${input.server}/auth/device/token`, {
          method: "POST",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            device_code: input.code,
            client_id: CLIENT_ID,
          }),
        })

        const parsed = yield* decodeSchema(DeviceToken, "Failed to decode device token response")(response)

        if (parsed instanceof DeviceTokenError) {
          return parsed.toPollResult()
        }

        const tokenSuccess = parsed as DeviceTokenSuccess
        const user = yield* fetchUser(input.server, tokenSuccess.accessToken)

        const now = yield* Clock.currentTimeMillis
        const expiry = now + tokenSuccess.expiresIn * 1000

        yield* repo.persistAccount({
          id: user.id,
          email: user.email,
          url: input.server,
          accessToken: tokenSuccess.accessToken as AccessToken,
          refreshToken: tokenSuccess.refreshToken as RefreshToken,
          expiry,
        })

        return new PollSuccess({ email: user.email })
      })

    const remove = (accountID: AccountID) =>
      repo.remove(accountID).pipe(
        Effect.catchTag("AccountRepoError", (e) =>
          Effect.fail(serviceError("Failed to remove account", e.cause)),
        ),
      )

    const use = (accountID: AccountID, workspaceID: Option.Option<WorkspaceID>) =>
      repo.use(accountID, workspaceID).pipe(
        Effect.catchTag("AccountRepoError", (e) =>
          Effect.fail(serviceError("Failed to switch workspace", e.cause)),
        ),
      )


    return {
      active,
      list,
      workspacesByAccount,
      remove,
      use,
      workspaces,
      token,
      login,
      poll,
      refreshToken,
    } as unknown as AccountServiceInterface
  }),
)

export const defaultLayer = layer.pipe(
  Layer.provide(AccountRepo.defaultLayer),
)
