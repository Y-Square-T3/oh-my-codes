import { Effect } from "effect"
import { AuditService } from "../../features/audit"
import * as AuditRepo from "../../features/audit/repo"
import * as Db from "../../features/database"
import { log } from "../../features/log/logger"
import type { TokenUsageEvent } from "../../features/audit/service"

export type EventInput = {
  event: {
    type: string
    properties?: unknown
  }
}

let cachedService: AuditService | null = null

const getOrCreateAuditService = async (): Promise<AuditService | null> => {
  if (cachedService) return cachedService

  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const db = yield* Db.Database
      yield* db.migrate()
      const repo = yield* AuditRepo.Service
      return new AuditService(repo, db)
    }).pipe(Effect.provide(AuditRepo.defaultLayer), Effect.provide(Db.defaultLayer)),
  )

  if (exit._tag === "Failure") {
    log("[audit-token-tracker] Failed to create service", { error: exit.cause })
    return null
  }

  cachedService = exit.value as AuditService
  return cachedService
}

const isCompactionAgent = (agent: string | undefined | null): boolean => {
  if (!agent) return false
  return agent.toLowerCase() === "compaction"
}

export const recordTokenUsage = async (input: EventInput): Promise<void> => {
  if (input.event?.type !== "message.updated") return

  const properties = input.event.properties as
    | {
        info?: {
          role?: string
          sessionID?: string
          messageID?: string
          id?: string
          agent?: string
          providerID?: string
          modelID?: string
          finish?: boolean
          tokens?: {
            input?: number
            output?: number
            reasoning?: number
            cache?: { read?: number; write?: number }
          }
        }
      }
    | undefined

  const info = properties?.info
  if (!info) return
  if (info.role !== "assistant" || !info.finish) return
  if (!info.sessionID || !info.tokens) return
  if (isCompactionAgent(info.agent)) return

  const service = await getOrCreateAuditService()
  if (!service) return

  const hasAccount = await Effect.runPromise(service.hasActiveAccount())
  if (!hasAccount) return

  const tokenUsageEvent: TokenUsageEvent = {
    sessionID: info.sessionID,
    messageID: info.id ?? info.messageID ?? "",
    agent: info.agent ?? null,
    providerID: info.providerID ?? "unknown",
    modelID: info.modelID ?? "unknown",
    inputTokens: info.tokens.input ?? 0,
    outputTokens: info.tokens.output ?? 0,
    reasoningTokens: info.tokens.reasoning ?? 0,
    cacheReadTokens: info.tokens.cache?.read ?? 0,
    cacheWriteTokens: info.tokens.cache?.write ?? 0,
    recordedAt: Date.now(),
  }

  await Effect.runPromise(service.recordTokenUsage(tokenUsageEvent)).catch((err) => {
    log("[audit-token-tracker] Failed to record token usage", { error: err })
  })
}
