import { Effect } from "effect"

import * as Db from "../../features/database"
import * as AuditRepo from "../../features/audit/repo"
import { AuditService } from "../../features/audit/service"

export interface TokenUsagesOptions {
  action: "status" | "push" | undefined
  json: boolean
  dbPath?: string
}

interface StatusResult {
  unpushedCount: number
  hasActiveAccount: boolean
}

interface PushResultOutput {
  pushedCount: number
  failedCount: number
  totalPushed: number
  totalBatches: number
}

const BATCH_SIZE = 20

async function getService(customPath?: string) {
  const dbLayer = customPath ? Db.makeLayer(customPath) : Db.defaultLayer

  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const db = yield* Db.Database
      yield* db.migrate()
      const repo = yield* AuditRepo.Service
      return new AuditService(repo, db)
    }).pipe(Effect.provide(AuditRepo.defaultLayer), Effect.provide(dbLayer)),
  )

  if (exit._tag === "Failure") {
    throw new Error(`Failed to create audit service: ${exit.cause}`)
  }

  return exit.value
}

async function runStatus(
  service: AuditService,
  json: boolean,
): Promise<number> {
  const [unpushedCount, hasActiveAccount] = await Effect.runPromise(
    Effect.all([service.countUnpushed(), service.hasActiveAccount()]),
  )

  if (json) {
    const output: StatusResult = { unpushedCount, hasActiveAccount }
    console.log(JSON.stringify(output, null, 2))
  } else {
    console.log(`Token Usages Status`)
    console.log(`  Unpushed records: ${unpushedCount}`)
    console.log(`  Active account: ${hasActiveAccount ? "yes" : "no"}`)
  }

  return 0
}

async function runPush(service: AuditService, json: boolean): Promise<number> {
  let totalPushed = 0
  let totalBatches = 0
  let lastFailed = false

  while (true) {
    const result = await Effect.runPromise(service.pushBatch(BATCH_SIZE, 30))

    totalBatches++
    totalPushed += result.pushedCount

    if (result.pushedCount < BATCH_SIZE) {
      lastFailed = result.failedCount > 0
      break
    }

    if (result.pushedCount === 0) {
      break
    }
  }

  if (json) {
    const output: PushResultOutput = {
      pushedCount: totalPushed,
      failedCount: lastFailed ? 1 : 0,
      totalPushed,
      totalBatches,
    }
    console.log(JSON.stringify(output, null, 2))
  } else {
    console.log(`Token Usages Push`)
    console.log(`  Total pushed: ${totalPushed}`)
    console.log(`  Batches: ${totalBatches}`)
  }

  return 0
}

export async function runTokenUsages(
  options: TokenUsagesOptions,
): Promise<number> {
  const { action, json, dbPath } = options

  try {
    const service = await getService(dbPath)

    if (!action || action === "status") {
      return runStatus(service, json)
    }

    if (action === "push") {
      return runPush(service, json)
    }

    console.error(`Unknown action: ${action}`)
    return 1
  } catch (err) {
    if (json) {
      console.log(JSON.stringify({ error: String(err) }, null, 2))
    } else {
      console.error(`Error: ${err}`)
    }
    return 1
  }
}
