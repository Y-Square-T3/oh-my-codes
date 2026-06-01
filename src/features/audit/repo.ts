import { Context, Effect, Layer } from "effect"
import { and, eq, lte, sql } from "drizzle-orm"

import * as Db from "../database"
import { AuditRecordID, type AuditRepoError, type TokenUsageRecord } from "./schema"
import { tokenUsages } from "../database/schema"

export interface AuditRepoService {
  readonly upsert: (record: TokenUsageRecord) => Effect.Effect<void, AuditRepoError>
  readonly fetchUnpushed: (limit: number) => Effect.Effect<TokenUsageRecord[], AuditRepoError>
  readonly markPushed: (ids: string[]) => Effect.Effect<void, AuditRepoError>
  readonly cleanupOldRecords: (beforeTimestamp: number) => Effect.Effect<number, AuditRepoError>
  readonly countUnpushed: () => Effect.Effect<number, AuditRepoError>
  readonly hasActiveAccount: () => Effect.Effect<boolean, AuditRepoError>
}

export const Service = Context.GenericTag<AuditRepoService>("@audit/Repo")

const mapAuditRepoError = (message: string) =>
  Effect.catchAll((cause: unknown) => {
    const err = cause instanceof Error ? cause : new Error(String(cause))
    return Effect.fail(Object.assign(err, { _tag: "AuditRepoError", message }) as AuditRepoError)
  })

const decodeRecord = (row: {
  id: string
  recordedAt: number
  sessionID: string
  messageID: string
  agent: string | null
  providerID: string
  modelID: string
  inputTokens: number
  outputTokens: number
  reasoningTokens: number
  cacheReadTokens: number
  cacheWriteTokens: number
  pushed: number
  createdAt: number
}): TokenUsageRecord => ({
  id: row.id as AuditRecordID,
  recordedAt: row.recordedAt,
  sessionID: row.sessionID,
  messageID: row.messageID,
  agent: row.agent,
  providerID: row.providerID,
  modelID: row.modelID,
  inputTokens: row.inputTokens,
  outputTokens: row.outputTokens,
  reasoningTokens: row.reasoningTokens,
  cacheReadTokens: row.cacheReadTokens,
  cacheWriteTokens: row.cacheWriteTokens,
  pushed: row.pushed === 1,
  createdAt: row.createdAt,
})

export const layer = Layer.effect(
  Service,
  Effect.gen(function* () {
    const database = yield* Db.Database
    yield* database.migrate()

    const upsert = (record: TokenUsageRecord) =>
      Effect.promise(() =>
        database.db
          .insert(tokenUsages)
          .values({
            id: record.id,
            recordedAt: record.recordedAt,
            sessionID: record.sessionID,
            messageID: record.messageID,
            agent: record.agent,
            providerID: record.providerID,
            modelID: record.modelID,
            inputTokens: record.inputTokens,
            outputTokens: record.outputTokens,
            reasoningTokens: record.reasoningTokens,
            cacheReadTokens: record.cacheReadTokens,
            cacheWriteTokens: record.cacheWriteTokens,
            pushed: record.pushed ? 1 : 0,
            createdAt: record.createdAt,
          })
          .onConflictDoUpdate({
            target: tokenUsages.messageID,
            set: {
              recordedAt: record.recordedAt,
              sessionID: record.sessionID,
              agent: record.agent,
              providerID: record.providerID,
              modelID: record.modelID,
              inputTokens: record.inputTokens,
              outputTokens: record.outputTokens,
              reasoningTokens: record.reasoningTokens,
              cacheReadTokens: record.cacheReadTokens,
              cacheWriteTokens: record.cacheWriteTokens,
              pushed: record.pushed ? 1 : 0,
              createdAt: record.createdAt,
            },
          }),
      ).pipe(mapAuditRepoError("upsert"))

    const fetchUnpushed = (limit: number) =>
      Effect.try({
        try: () =>
          database.db
            .select()
            .from(tokenUsages)
            .where(eq(tokenUsages.pushed, 0))
            .limit(limit)
            .orderBy(tokenUsages.recordedAt)
            .all(),
        catch: (cause) => new Error("Failed to fetch unpushed records", { cause }),
      }).pipe(
        Effect.map((rows) => rows.map(decodeRecord)),
        mapAuditRepoError("fetchUnpushed"),
      )

    const markPushed = (ids: string[]) =>
      Effect.promise(async () => {
        if (ids.length === 0) return
        await database.db
          .update(tokenUsages)
          .set({ pushed: 1 })
          .where(
            sql`${tokenUsages.id} IN (${sql.join(
              ids.map((id) => sql`${id}`),
              sql`, `,
            )})`,
          )
      }).pipe(mapAuditRepoError("markPushed"))

    const cleanupOldRecords = (beforeTimestamp: number) =>
      Effect.try({
        try: () => {
          database.db
            .delete(tokenUsages)
            .where(and(eq(tokenUsages.pushed, 1), lte(tokenUsages.recordedAt, beforeTimestamp)))
          return database.db
            .select({ count: sql<number>`count(*)` })
            .from(tokenUsages)
            .where(eq(tokenUsages.pushed, 1))
            .all()
        },
        catch: (cause) => new Error("Failed to cleanup old records", { cause }),
      }).pipe(
        Effect.map((rows) => rows[0]?.count ?? 0),
        mapAuditRepoError("cleanupOldRecords"),
      )

    const countUnpushed = () =>
      Effect.try({
        try: () =>
          database.db
            .select({ count: sql<number>`count(*)` })
            .from(tokenUsages)
            .where(eq(tokenUsages.pushed, 0))
            .all(),
        catch: (cause) => new Error("Failed to count unpushed records", { cause }),
      }).pipe(
        Effect.map((rows) => rows[0]?.count ?? 0),
        mapAuditRepoError("countUnpushed"),
      )

    const hasActiveAccount = () =>
      Effect.try({
        try: () =>
          database.db
            .select()
            .from(Db.schema.accountState)
            .where(eq(Db.schema.accountState.id, 1))
            .get(),
        catch: (cause) => new Error("Failed to check active account", { cause }),
      }).pipe(
        Effect.map((state) => state?.activeAccountId != null),
        mapAuditRepoError("hasActiveAccount"),
      )

    return {
      upsert,
      fetchUnpushed,
      markPushed,
      cleanupOldRecords,
      countUnpushed,
      hasActiveAccount,
    }
  }),
)

export const defaultLayer = layer.pipe(Layer.provide(Db.defaultLayer))
