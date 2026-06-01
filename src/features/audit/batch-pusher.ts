import type { AuditServiceInterface } from "./service"
import { AuditService } from "./service"
import * as AuditRepo from "./repo"
import * as Db from "../database"
import { Effect } from "effect"
import { log } from "../log/logger"
import { PluginContext } from "../../types"

const DefaultConfig = {
  push_interval_ms: 30_000,
  batch_size: 20,
  retention_days: 30,
  disabled: false,
}

export interface AuditBatchPusher {
  readonly start: () => void
  readonly stop: () => Promise<void>
  readonly forcePush: () => Promise<void>
}

export function createAuditBatchPusher(
  _ctx: PluginContext,
  config: typeof DefaultConfig = DefaultConfig,
): AuditBatchPusher {
  let timer: ReturnType<typeof setInterval> | null = null
  let service: AuditServiceInterface | null = null

  const getOrCreateService = async (): Promise<AuditServiceInterface> => {
    if (service) return service

    const exit = await Effect.runPromiseExit(
      Effect.gen(function* () {
        const db = yield* Db.Database
        yield* db.migrate()
        const repo = yield* AuditRepo.Service
        return new AuditService(repo, db)
      }).pipe(Effect.provide(AuditRepo.defaultLayer), Effect.provide(Db.defaultLayer)),
    )

    if (exit._tag === "Failure") {
      log("[audit] Failed to create service", { error: exit.cause })
      throw new Error("Audit service not available")
    }

    service = exit.value as AuditServiceInterface
    return service
  }

  const pushBatch = async (): Promise<void> => {
    const svc = await getOrCreateService()
    const batchSize = config.batch_size ?? 20
    const retentionDays = config.retention_days ?? 30

    const result = await Effect.runPromise(svc.pushBatch(batchSize, retentionDays))

    if (result.pushedCount > 0 || result.failedCount > 0) {
      log("[audit] Push batch result", { result })
    }
  }

  const start = (): void => {
    if (config.disabled) {
      log("[audit] Disabled by config, not starting batch pusher")
      return
    }

    const intervalMs = config.push_interval_ms ?? 30_000
    log("[audit] Starting batch pusher", { intervalMs })

    timer = setInterval(() => {
      pushBatch().catch((err) => {
        log("[audit] Tick error", { error: err })
      })
    }, intervalMs)
  }

  const stop = async (): Promise<void> => {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
    log("[audit] Batch pusher stopped")

    if (service) {
      try {
        const batchSize = config.batch_size ?? 20
        const retentionDays = config.retention_days ?? 30
        await Effect.runPromise(service.pushBatch(batchSize, retentionDays))
      } catch (err) {
        log("[audit] Final push failed", { error: err })
      }
    }
  }

  const forcePush = async (): Promise<void> => {
    const svc = await getOrCreateService()
    const batchSize = config.batch_size ?? 20
    const retentionDays = config.retention_days ?? 30

    const result = await Effect.runPromise(svc.pushBatch(batchSize, retentionDays))
    log("[audit] forcePush result", { result })
  }

  return { start, stop, forcePush }
}
