import { Cause, Effect, Option } from "effect"
import { tool, type ToolDefinition } from "@opencode-ai/plugin"
import type { PluginContext } from "../types"
import * as Account from "../features/account"
import * as Model from "../features/model"
import { AccountID, WorkspaceID } from "../features/account/schema"
import { log } from "../features/log/logger"

type SwitchToolArgs = {
  id?: string
}

type WorkspaceEntry = {
  index: number
  name: string
  email: string
  url: string
  accountId: AccountID
  workspaceId: WorkspaceID
  isActive: boolean
}

async function getWorkspaceEntries(): Promise<Result<WorkspaceEntry[]>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      const groups = yield* accountSvc.workspacesByAccount()
      const active = yield* accountSvc.active().pipe(Effect.catchAll(() => Effect.succeed(Option.none())))
      const activeAccountId = Option.map(active, (a) => a.id)

      const entries: WorkspaceEntry[] = []
      let index = 0

      for (const group of groups) {
        for (const ws of group.workspaces) {
          index++
          const isActive =
            Option.isSome(activeAccountId) &&
            activeAccountId.value === group.account.id &&
            group.account.activeWorkspaceId === ws.id
          entries.push({
            index,
            name: ws.name,
            email: group.account.email,
            url: group.account.url,
            accountId: group.account.id as AccountID,
            workspaceId: ws.id,
            isActive,
          })
        }
      }

      return entries
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

function formatWorkspaceList(entries: WorkspaceEntry[]): string {
  if (entries.length === 0) {
    return "No workspaces found. Please login first with /omc-login <url>."
  }

  const lines = entries.map((e) => {
    const marker = e.isActive ? " ← active" : ""
    return `${e.index}. ${e.name} [${e.workspaceId}]${marker}`
  })

  return `Available workspaces:\n${lines.join("\n")}\n\nSwitch by number or workspace ID:\n/omc-switch <number>`
}

async function refreshModels(): Promise<string | null> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const modelSvc = yield* Model.Service
      const result = yield* modelSvc.refresh()
      return `Refreshed ${result.providers} providers, ${result.models} models`
    }).pipe(Effect.provide(Model.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return exit.value
  }
  log("[omc-switch] Model refresh failed", { error: Cause.pretty(exit.cause) })
  return null
}

async function switchTo(entry: WorkspaceEntry): Promise<Result<string>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      yield* accountSvc.use(entry.accountId, Option.some(entry.workspaceId))
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (!exit._tag || exit._tag === "Success") {
    return { ok: true, value: `Switched to ${entry.name} (${entry.email})` }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

type Result<T> = { ok: true; value: T } | { ok: false; error: string }

export function createSwitchTool(ctx: PluginContext): ToolDefinition {
  return tool({
    description:
      "Switch between OMC workspaces. Run without arguments to list all workspaces with numbers, then use /omc-switch <number> or <workspace-id> to switch.",
    args: {
      id: tool.schema
        .string()
        .optional()
        .describe("Workspace number (from the list) or workspace ID to switch to. Leave empty to list all workspaces."),
    },
    async execute(args: SwitchToolArgs): Promise<string> {
      const entriesResult = await getWorkspaceEntries()
      if (!entriesResult.ok) {
        return `Failed to load workspaces: ${entriesResult.error}`
      }

      const entries = entriesResult.value

      if (!args.id || args.id.trim() === "") {
        return formatWorkspaceList(entries)
      }

      const trimmed = args.id.trim()
      const asNumber = Number(trimmed)
      let selected: WorkspaceEntry | undefined

      if (!isNaN(asNumber)) {
        selected = entries.find((e) => e.index === asNumber)
      }

      if (!selected) {
        selected = entries.find((e) => e.workspaceId === trimmed)
      }

      if (!selected) {
        return `"${trimmed}" is not a valid workspace number or ID.\n\n${formatWorkspaceList(entries)}`
      }

      const switchResult = await switchTo(selected)

      if (!switchResult.ok) {
        return `Failed to switch workspace: ${switchResult.error}`
      }

      const refreshMsg = await refreshModels()

      const parts = [switchResult.value]
      if (refreshMsg) {
        parts.push(refreshMsg)
      }

      return parts.join("\n\n")
    },
  })
}
