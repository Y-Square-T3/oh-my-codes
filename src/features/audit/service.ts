import { Effect } from "effect"
import { eq } from "drizzle-orm"

import * as Db from "../database"
import * as AuditRepo from "./repo"
import { type AuditError, type AuditRecordID, type PushResult, type TokenUsageRecord } from "./schema"
import { log } from "../log/logger"

export interface AuditServiceInterface {
  readonly recordTokenUsage: (event: TokenUsageEvent) => Effect.Effect<void, AuditError>
  readonly pushBatch: (batchSize: number, retentionDays: number) => Effect.Effect<PushResult, AuditError>
  readonly cleanup: (retentionDays: number) => Effect.Effect<number, AuditError>
  readonly hasActiveAccount: () => Effect.Effect<boolean, AuditError>
  readonly countUnpushed: () => Effect.Effect<number, AuditError>
}

export interface TokenUsageEvent {
  readonly sessionID: string
  readonly messageID: string
  readonly agent: string | null
  readonly providerID: string
  readonly modelID: string
  readonly inputTokens: number
  readonly outputTokens: number
  readonly reasoningTokens: number
  readonly cacheReadTokens: number
  readonly cacheWriteTokens: number
  readonly recordedAt: number
}

const toPayload = (
  records: TokenUsageRecord[],
): Array<{
  model: string
  prompt_tokens: number
  completion_tokens: number
  reasoning_tokens: number
  request_id: string
  session_id: string | null
  created_at: string
}> =>
  records.map((r) => ({
    model: `${r.providerID}/${r.modelID}`,
    prompt_tokens: r.inputTokens,
    completion_tokens: r.outputTokens,
    reasoning_tokens: r.reasoningTokens,
    request_id: r.messageID,
    session_id: r.sessionID,
    created_at: new Date(r.recordedAt).toISOString(),
  }))

function createAuditService(repo: AuditRepo.AuditRepoService, db: Db.DatabaseService): AuditServiceInterface {
  const recordTokenUsage = (event: TokenUsageEvent): Effect.Effect<void, AuditError> =>
    Effect.gen(function* () {
      const id = crypto.randomUUID() as AuditRecordID
      const now = Date.now()
      const record: TokenUsageRecord = {
        id,
        recordedAt: event.recordedAt,
        sessionID: event.sessionID,
        messageID: event.messageID,
        agent: event.agent,
        providerID: event.providerID,
        modelID: event.modelID,
        inputTokens: event.inputTokens,
        outputTokens: event.outputTokens,
        reasoningTokens: event.reasoningTokens,
        cacheReadTokens: event.cacheReadTokens,
        cacheWriteTokens: event.cacheWriteTokens,
        pushed: false,
        createdAt: now,
      }
      yield* repo.upsert(record)
    }).pipe(
      Effect.catchAll((cause) => {
        const err = cause instanceof Error ? cause : new Error(String(cause))
        return Effect.fail(
          Object.assign(err, {
            _tag: "AuditServiceError" as const,
            message: "Failed to record token usage",
            cause,
          }),
        )
      }),
    )

  const pushBatch = (batchSize: number, retentionDays: number): Effect.Effect<PushResult, AuditError> => {
    return Effect.gen(function* () {
      const records = yield* repo.fetchUnpushed(batchSize)
      if (records.length === 0) {
        return { pushedCount: 0, failedCount: 0, ids: [] } as PushResult
      }

      yield* db.migrate()

      const state = yield* Effect.tryPromise({
        try: () =>
          db.db.query.accountState.findFirst({
            where: eq(Db.schema.accountState.id, 1),
          }),
        catch: (cause) => new Error("Failed to fetch account state", { cause }),
      })

      if (!state?.activeAccountId) {
        log("[audit] No active account, skipping batch push")
        return { pushedCount: 0, failedCount: 0, ids: [] } as PushResult
      }

      const workspaceId = state.activeWorkspaceId ?? undefined

      const account = yield* Effect.tryPromise({
        try: () =>
          db.db.query.accounts.findFirst({
            where: eq(Db.schema.accounts.id, state.activeAccountId!),
          }),
        catch: (cause) => new Error("Failed to fetch account", { cause }),
      })

      if (!account) {
        log("[audit] Account not found, skipping batch push")
        return { pushedCount: 0, failedCount: 0, ids: [] } as PushResult
      }

      const loggedInModels = yield* Effect.tryPromise({
        try: () =>
          db.db.query.modelRecords.findMany({
            where: eq(Db.schema.modelRecords.accountId, state.activeAccountId!),
          }),
        catch: (cause) => new Error("Failed to fetch logged-in models", { cause }),
      })

      const loggedInModelIds = new Set(loggedInModels.map((m) => m.id))
      const filteredRecords = records.filter((r) => !loggedInModelIds.has(r.modelID))
      const filteredCount = records.length - filteredRecords.length

      if (filteredCount > 0) {
        log("[audit] Filtered out records with logged-in model IDs", {
          filteredCount,
        })
      }

      if (filteredRecords.length === 0) {
        return { pushedCount: 0, failedCount: 0, ids: [] } as PushResult
      }

      const payload = toPayload(filteredRecords)

      const pushUrl = `${account.url}/api/v2/token-usages/batch`
      let pushSuccess = false
      try {
        const headers: Record<string, string> = {
          "Content-Type": "application/json",
          Authorization: `Bearer ${account.accessToken}`,
        }
        if (workspaceId) {
          headers["x-workspace-id"] = workspaceId
        }
        const response = yield* Effect.tryPromise({
          try: () =>
            fetch(pushUrl, {
              method: "POST",
              headers,
              body: JSON.stringify(payload),
            }),
          catch: (cause) => new Error("HTTP request failed", { cause }),
        })

        if (!response.ok) {
          throw new Error(`HTTP ${response.status} from ${pushUrl}`)
        }
        pushSuccess = true
      } catch (err) {
        log("[audit] Batch push HTTP failed", { error: err })
      }

      if (pushSuccess) {
        yield* repo.markPushed(records.map((rec) => rec.id))
        log("[audit] Batch push succeeded", { count: filteredRecords.length })

        const beforeTimestamp = Date.now() - retentionDays * 24 * 60 * 60 * 1000
        const deleted = yield* repo.cleanupOldRecords(beforeTimestamp)
        if (deleted > 0) {
          log("[audit] Cleanup deleted records", { count: deleted })
        }
      }

      return {
        pushedCount: pushSuccess ? filteredRecords.length : 0,
        failedCount: pushSuccess ? 0 : filteredRecords.length,
        ids: filteredRecords.map((rec) => rec.id),
      } as PushResult
    }).pipe(
      Effect.catchAll((cause) => {
        log("[audit] Batch push failed", { error: cause })
        return Effect.succeed({
          pushedCount: 0,
          failedCount: 0,
          ids: [],
        } as PushResult)
      }),
    )
  }

  const cleanup = (retentionDays: number): Effect.Effect<number, AuditError> => {
    const beforeTimestamp = Date.now() - retentionDays * 24 * 60 * 60 * 1000
    return repo.cleanupOldRecords(beforeTimestamp).pipe(
      Effect.catchAll((cause) => {
        const err = cause instanceof Error ? cause : new Error(String(cause))
        return Effect.fail(
          Object.assign(err, {
            _tag: "AuditServiceError" as const,
            message: "Failed to cleanup old records",
            cause,
          }),
        )
      }),
    )
  }

  const hasActiveAccount = (): Effect.Effect<boolean, AuditError> =>
    repo.hasActiveAccount().pipe(Effect.catchAll(() => Effect.succeed(false)))

  const countUnpushed = (): Effect.Effect<number, AuditError> =>
    repo.countUnpushed().pipe(
      Effect.catchAll((cause) => {
        log("[audit] countUnpushed failed", { error: cause })
        return Effect.succeed(0)
      }),
    )

  return {
    recordTokenUsage,
    pushBatch,
    cleanup,
    hasActiveAccount,
    countUnpushed,
  }
}

export class AuditService {
  constructor(
    private readonly repo: AuditRepo.AuditRepoService,
    private readonly db: Db.DatabaseService,
  ) {}

  recordTokenUsage = (event: TokenUsageEvent): Effect.Effect<void, AuditError> =>
    createAuditService(this.repo, this.db).recordTokenUsage(event)

  pushBatch = (batchSize: number, retentionDays: number): Effect.Effect<PushResult, AuditError> =>
    createAuditService(this.repo, this.db).pushBatch(batchSize, retentionDays)

  cleanup = (retentionDays: number): Effect.Effect<number, AuditError> =>
    createAuditService(this.repo, this.db).cleanup(retentionDays)

  hasActiveAccount = (): Effect.Effect<boolean, AuditError> => createAuditService(this.repo, this.db).hasActiveAccount()

  countUnpushed = (): Effect.Effect<number, AuditError> => createAuditService(this.repo, this.db).countUnpushed()
}

export const Service = AuditRepo.Service
