import { Context, Effect, Layer, Option } from "effect"

import * as Db from "../database"
import {
  AccountID,
  AccessToken,
  RefreshToken,
  WorkspaceID,
  AccountInfo as AccountInfoSchema,
  AccountRepoError,
} from "./schema"

export interface AccountRow {
  id: AccountID
  email: string
  url: string
  access_token: AccessToken
  refresh_token: RefreshToken
  token_expiry: number | null
}

export interface AccountStateRow {
  id: number
  active_account_id: AccountID | null
  active_workspace_id: WorkspaceID | null
}

export interface AccountRepoService {
  readonly active: () => Effect.Effect<Option.Option<AccountInfoSchema>, AccountRepoError>
  readonly list: () => Effect.Effect<AccountInfoSchema[], AccountRepoError>
  readonly remove: (accountID: AccountID) => Effect.Effect<void, AccountRepoError>
  readonly use: (accountID: AccountID, workspaceID: Option.Option<WorkspaceID>) => Effect.Effect<void, AccountRepoError>
  readonly getRow: (accountID: AccountID) => Effect.Effect<Option.Option<AccountRow>, AccountRepoError>
  readonly persistToken: (input: {
    accountID: AccountID
    accessToken: AccessToken
    refreshToken: RefreshToken
    expiry: Option.Option<number>
  }) => Effect.Effect<void, AccountRepoError>
  readonly persistAccount: (input: {
    id: AccountID
    email: string
    url: string
    accessToken: AccessToken
    refreshToken: RefreshToken
    expiry: number
  }) => Effect.Effect<void, AccountRepoError>
}

export const Service = Context.GenericTag<AccountRepoService>("@account/AccountRepo")

const decodeAccountInfo = (row: { id: AccountID; email: string; url: string }): AccountInfoSchema =>
  new AccountInfoSchema({
    id: row.id,
    email: row.email,
    url: row.url,
    active_workspace_id: null,
  })

const runOneTyped = <T>(db: Db.DatabaseService, sql: string, params?: unknown[]): Effect.Effect<Option.Option<T>, Db.DatabaseQueryError> =>
  db.runOne(sql, params as string[]).pipe(
    Effect.map((opt: Option.Option<unknown>) => opt as Option.Option<T>),
  )

const runAllTyped = <T>(db: Db.DatabaseService, sql: string, params?: unknown[]): Effect.Effect<T[], Db.DatabaseQueryError> =>
  db.runAll(sql, params as string[]).pipe(
    Effect.map((arr: unknown[]) => arr as T[]),
  )

const mapDbError = (message: string) =>
  Effect.catchAll((cause: Db.DatabaseQueryError) =>
    Effect.fail(new AccountRepoError({ message, cause: cause.cause }))
  )

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const database = yield* Db.Database

    const active = () =>
      Effect.gen(function* () {
        const stateOpt = yield* runOneTyped<AccountStateRow>(database, "SELECT * FROM account_state WHERE id = 1").pipe(mapDbError("Failed to read account state"))
        if (Option.isNone(stateOpt)) return Option.none()

        const state = stateOpt.value
        if (!state.active_account_id) return Option.none()

        const accountOpt = yield* runOneTyped<{ id: AccountID; email: string; url: string }>(
          database,
          "SELECT id, email, url FROM accounts WHERE id = ?",
          [state.active_account_id],
        ).pipe(mapDbError("Failed to read account"))

        if (Option.isNone(accountOpt)) return Option.none()

        return Option.some(new AccountInfoSchema({
          id: accountOpt.value.id,
          email: accountOpt.value.email,
          url: accountOpt.value.url,
          active_workspace_id: state.active_workspace_id,
        }))
      })

    const list = () =>
      runAllTyped<{ id: AccountID; email: string; url: string }>(
        database,
        "SELECT id, email, url FROM accounts ORDER BY email",
      ).pipe(
        Effect.map((rows) => rows.map(decodeAccountInfo)),
        mapDbError("Failed to list accounts"),
      )

    const remove = (accountID: AccountID) =>
      Effect.gen(function* () {
        yield* database.run(
          "UPDATE account_state SET active_account_id = NULL, active_workspace_id = NULL WHERE active_account_id = ?",
          [accountID as string],
        ).pipe(mapDbError("Failed to update account state"))

        yield* database.run("DELETE FROM accounts WHERE id = ?", [accountID as string]).pipe(mapDbError("Failed to delete account"))
      })

    const use = (accountID: AccountID, workspaceID: Option.Option<WorkspaceID>) =>
      database.run(
        `INSERT INTO account_state (id, active_account_id, active_workspace_id)
         VALUES (1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET active_account_id = ?, active_workspace_id = ?`,
        [accountID as string, Option.getOrElse(workspaceID, () => null as WorkspaceID | null), accountID as string, Option.getOrElse(workspaceID, () => null as WorkspaceID | null)],
      ).pipe(
        Effect.asVoid,
        mapDbError("Failed to update active workspace"),
      )

    const getRow = (accountID: AccountID) =>
      runOneTyped<AccountRow>(database, "SELECT * FROM accounts WHERE id = ?", [accountID as string]).pipe(
        mapDbError("Failed to get account row"),
      )

    const persistToken = (input: {
      accountID: AccountID
      accessToken: AccessToken
      refreshToken: RefreshToken
      expiry: Option.Option<number>
    }) =>
      database.run(
        "UPDATE accounts SET access_token = ?, refresh_token = ?, token_expiry = ? WHERE id = ?",
        [input.accessToken as string, input.refreshToken as string, Option.getOrElse(input.expiry, () => null as number | null), input.accountID as string],
      ).pipe(
        Effect.asVoid,
        mapDbError("Failed to persist token"),
      )

    const persistAccount = (input: {
      id: AccountID
      email: string
      url: string
      accessToken: AccessToken
      refreshToken: RefreshToken
      expiry: number
    }) =>
      database.run(
        `INSERT INTO accounts (id, email, url, access_token, refresh_token, token_expiry)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET email = ?, url = ?, access_token = ?, refresh_token = ?, token_expiry = ?`,
        [
          input.id as string, input.email, input.url, input.accessToken as string, input.refreshToken as string, input.expiry,
          input.email, input.url, input.accessToken as string, input.refreshToken as string, input.expiry,
        ],
      ).pipe(
        Effect.asVoid,
        mapDbError("Failed to persist account"),
      )

    return {
      active,
      list,
      remove,
      use,
      getRow,
      persistToken,
      persistAccount,
    }
  }),
)

export const defaultLayer = layer.pipe(Layer.provide(Db.defaultLayer))
