import { Context, Effect, Layer, Option } from "effect"
import { asc, eq } from "drizzle-orm"

import * as Db from "../database"
import {
  AccessToken,
  AccountID,
  AccountInfo as AccountInfoSchema,
  AccountRepoError,
  RefreshToken,
  WorkspaceID,
} from "./schema"

export interface AccountRow {
  readonly id: AccountID
  readonly email: string
  readonly url: string
  readonly access_token: AccessToken
  readonly refresh_token: RefreshToken
  readonly token_expiry: number | null
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

const decodeAccountInfo = (
  row: { id: string; email: string; url: string },
  activeWorkspaceId: string | null,
): AccountInfoSchema =>
  new AccountInfoSchema({
    id: row.id as AccountID,
    email: row.email,
    url: row.url,
    activeWorkspaceId: activeWorkspaceId as WorkspaceID | null,
  })

const mapDbError = <A>(message: string) =>
  Effect.catchAll((cause: Db.DatabaseQueryError) => Effect.fail(new AccountRepoError({ message, cause: cause.cause })))

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const database = yield* Db.Database
    yield* database.migrate()

    const active = () =>
      Effect.gen(function* () {
        const stateOpt = yield* Effect.try({
          try: () =>
            database.db
              .select()
              .from(Db.schema.accountState)
              .where(eq(Db.schema.accountState.id, 1))
              .get(),
          catch: (cause) => new Db.DatabaseQueryError("Failed to read account state", cause),
        }).pipe(mapDbError("Failed to read account state"))

        if (!stateOpt) return Option.none()
        if (!stateOpt.activeAccountId) return Option.none()

        const activeAccountId = stateOpt.activeAccountId

        const accountOpt = yield* Effect.try({
          try: () =>
            database.db
              .select({
                id: Db.schema.accounts.id,
                email: Db.schema.accounts.email,
                url: Db.schema.accounts.url,
              })
              .from(Db.schema.accounts)
              .where(eq(Db.schema.accounts.id, activeAccountId))
              .get(),
          catch: (cause) => new Db.DatabaseQueryError("Failed to read account", cause),
        }).pipe(mapDbError("Failed to read account"))

        if (!accountOpt) return Option.none()

        return Option.some(decodeAccountInfo(accountOpt, stateOpt.activeWorkspaceId))
      })

    const list = () =>
      Effect.try({
        try: () =>
          database.db
            .select({
              id: Db.schema.accounts.id,
              email: Db.schema.accounts.email,
              url: Db.schema.accounts.url,
            })
            .from(Db.schema.accounts)
            .orderBy(asc(Db.schema.accounts.email))
            .all(),
        catch: (cause) => new Db.DatabaseQueryError("Failed to list accounts", cause),
      }).pipe(
        Effect.map((rows) => rows.map((row) => decodeAccountInfo(row, null))),
        mapDbError("Failed to list accounts"),
      )

    const remove = (accountID: AccountID) =>
      Effect.gen(function* () {
        yield* Effect.promise(() =>
          database.db
            .update(Db.schema.accountState)
            .set({
              activeAccountId: null,
              activeWorkspaceId: null,
            })
            .where(eq(Db.schema.accountState.activeAccountId, accountID)),
        ).pipe(mapDbError("Failed to update account state"))

        yield* Effect.promise(() =>
          database.db.delete(Db.schema.accounts).where(eq(Db.schema.accounts.id, accountID)),
        ).pipe(mapDbError("Failed to delete account"))
      })

    const use = (accountID: AccountID, workspaceID: Option.Option<WorkspaceID>) => {
      const wsId = Option.getOrElse(workspaceID, () => null as WorkspaceID | null)
      return Effect.promise(() =>
        database.db
          .insert(Db.schema.accountState)
          .values({
            id: 1,
            activeAccountId: accountID,
            activeWorkspaceId: wsId,
          })
          .onConflictDoUpdate({
            target: Db.schema.accountState.id,
            set: {
              activeAccountId: accountID,
              activeWorkspaceId: wsId,
            },
          }),
      ).pipe(Effect.asVoid, mapDbError("Failed to update active workspace"))
    }

    const getRow = (accountID: AccountID) =>
      Effect.try({
        try: () =>
          database.db
            .select()
            .from(Db.schema.accounts)
            .where(eq(Db.schema.accounts.id, accountID))
            .get(),
        catch: (cause) => new Db.DatabaseQueryError("Failed to get account row", cause),
      }).pipe(
        Effect.map((row) =>
          row
            ? Option.some({
                id: row.id as AccountID,
                email: row.email,
                url: row.url,
                access_token: row.accessToken as AccessToken,
                refresh_token: row.refreshToken as RefreshToken,
                token_expiry: row.tokenExpiry,
              } satisfies AccountRow)
            : Option.none(),
        ),
        mapDbError("Failed to get account row"),
      )

    const persistToken = (input: {
      accountID: AccountID
      accessToken: AccessToken
      refreshToken: RefreshToken
      expiry: Option.Option<number>
    }) =>
      Effect.promise(() =>
        database.db
          .update(Db.schema.accounts)
          .set({
            accessToken: input.accessToken,
            refreshToken: input.refreshToken,
            tokenExpiry: Option.getOrElse(input.expiry, () => null as number | null),
          })
          .where(eq(Db.schema.accounts.id, input.accountID)),
      ).pipe(Effect.asVoid, mapDbError("Failed to persist token"))

    const persistAccount = (input: {
      id: AccountID
      email: string
      url: string
      accessToken: AccessToken
      refreshToken: RefreshToken
      expiry: number
    }) =>
      Effect.promise(() =>
        database.db
          .insert(Db.schema.accounts)
          .values({
            id: input.id,
            email: input.email,
            url: input.url,
            accessToken: input.accessToken,
            refreshToken: input.refreshToken,
            tokenExpiry: input.expiry,
          })
          .onConflictDoUpdate({
            target: Db.schema.accounts.id,
            set: {
              email: input.email,
              url: input.url,
              accessToken: input.accessToken,
              refreshToken: input.refreshToken,
              tokenExpiry: input.expiry,
            },
          }),
      ).pipe(Effect.asVoid, mapDbError("Failed to persist account"))

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
