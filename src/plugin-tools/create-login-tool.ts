import { execFile } from "child_process"
import { Cause, Duration, Effect, Option } from "effect"
import { tool, type ToolDefinition } from "@opencode-ai/plugin"
import type { PluginContext } from "../types"
import * as Account from "../features/account"
import * as Model from "../features/model"
import { PollResult, PollSuccess, Login } from "../features/account/schema"
import { log } from "../features/log/logger"

type LoginToolArgs = {
  url?: string
}

async function showToast(ctx: PluginContext, message: string, variant: "success" | "error" | "warning"): Promise<void> {
  try {
    await ctx.client.tui.showToast({
      body: { message, variant },
    })
  } catch (err) {
    log("[omc-login] Failed to show toast", { error: err })
  }
}

async function runRefreshModels(): Promise<string | null> {
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
  const cause = Cause.pretty(exit.cause)
  log("[omc-login] Model refresh failed", { error: cause })
  return null
}

type Result<T> = { ok: true; value: T } | { ok: false; error: string }

async function runInitLogin(url: string): Promise<Result<Login>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      return yield* accountSvc.login(url)
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

async function runPoll(login: Login): Promise<Result<string>> {
  const maxAttempts = Math.ceil(login.expiry / Math.max(login.interval, 1))
  let attempts = 0

  const exit = await Effect.runPromiseExit(
    (Effect.gen(function* () {
      const accountSvc = yield* Account.Service

      let wait = login.interval
      let pollResult: PollResult | undefined

      while (!pollResult && attempts < maxAttempts) {
        attempts++
        yield* Effect.sleep(Duration.seconds(wait))
        const result = yield* accountSvc.poll(login)
        if (result._tag === "PollPending") continue
        if (result._tag === "PollSlow") {
          wait += 5
          continue
        }
        pollResult = result
      }

      if (!pollResult) {
        return yield* Effect.fail(new Error("Authentication timed out"))
      }

      if (pollResult._tag === "PollExpired" || pollResult._tag === "PollDenied" || pollResult._tag === "PollError") {
        return yield* Effect.fail(new Error(`Authentication failed: ${pollResult._tag}`))
      }

      return (pollResult as PollSuccess).email
    }) as any).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value as string }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

async function runSelectFirstWorkspace(): Promise<Result<boolean>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      const groups = yield* accountSvc.workspacesByAccount()

      if (groups.length === 0) {
        return false
      }

      const firstGroup = groups[0]
      if (firstGroup.workspaces.length === 0) {
        return false
      }

      const firstWorkspace = firstGroup.workspaces[0]
      yield* accountSvc.use(firstGroup.account.id, Option.some(firstWorkspace.id))
      return true
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

async function runGetWorkspacesList(): Promise<Result<{ list: string; totalCount: number }>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      const groups = yield* accountSvc.workspacesByAccount()

      const active = yield* accountSvc.active().pipe(Effect.catchAll(() => Effect.succeed(Option.none())))
      const activeAccountId = Option.map(active, (a) => a.id)

      const lines: string[] = []
      let count = 0

      for (const group of groups) {
        for (const ws of group.workspaces) {
          count++
          const isActive =
            Option.isSome(activeAccountId) &&
            activeAccountId.value === group.account.id &&
            group.account.activeWorkspaceId === ws.id
          const marker = isActive ? " (active)" : ""
          lines.push(`${count}. ${ws.name} (${group.account.email}, ${group.account.url})${marker}`)
        }
      }

      if (count === 0) {
        return { list: "No workspaces found.", totalCount: 0 }
      }

      return {
        list: `Available workspaces:\n${lines.join("\n")}`,
        totalCount: count,
      }
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value }
  }
  return { ok: false, error: Cause.pretty(exit.cause) }
}

export function createLoginTool(ctx: PluginContext): ToolDefinition {
  return tool({
    description:
      "Login to an OMC (oh-my-codes) account using device code flow. Provide the server URL to authenticate. This will open your browser for authentication.",
    args: {
      url: tool.schema
        .string()
        .describe("The OMC server URL (e.g. https://server.omc.ai). Required for login."),
    },
    async execute(args: LoginToolArgs): Promise<string> {
      const serverUrl = args.url
      if (!serverUrl) {
        return "Please provide a server URL. Usage: /omc-login <url>"
      }

      await showToast(ctx, "Starting OMC login...", "warning")

      const loginInitResult = await runInitLogin(serverUrl)
      if (!loginInitResult.ok) {
        await showToast(ctx, "Login failed", "error")
        return `Login failed: ${loginInitResult.error}`
      }

      const loginInfo = loginInitResult.value

      if (process.platform === "win32") {
        void execFile("cmd", ["/c", "start", "", loginInfo.url])
      } else if (process.platform === "darwin") {
        void execFile("open", [loginInfo.url])
      } else {
        void execFile("xdg-open", [loginInfo.url])
      }

      log("[omc-login] Browser opened", { url: loginInfo.url, code: loginInfo.user })

      await showToast(ctx, "Open your browser to complete authentication", "warning")

      try {
        await ctx.client.app.log({
          body: {
            service: "oh-my-codes",
            level: "info",
            message: `Authentication URL: ${loginInfo.url}\nAuth code: ${loginInfo.user}`,
          },
        })
      } catch (err) {
        log("[omc-login] Failed to log URL", { error: err })
      }

      const pollResult = await runPoll(loginInfo)
      if (!pollResult.ok) {
        await showToast(ctx, "Login failed", "error")
        return `Authentication failed: ${pollResult.error}`
      }

      const email = pollResult.value
      await showToast(ctx, `Logged in as ${email}`, "success")

      const workspaceSelectResult = await runSelectFirstWorkspace()
      if (!workspaceSelectResult.ok) {
        log("[omc-login] Workspace auto-select failed", { error: workspaceSelectResult.error })
      }

      const refreshMsg = await runRefreshModels()

      const listResult = await runGetWorkspacesList()
      if (!listResult.ok) {
        log("[omc-login] Failed to get workspace list", { error: listResult.error })
      }

      const parts: string[] = [
        `Successfully logged in as ${email}`,
      ]

      if (refreshMsg) {
        parts.push(refreshMsg)
      }

      if (listResult.ok && listResult.value.totalCount > 1) {
        parts.push("\nYou have multiple workspaces. To switch:")
        parts.push(listResult.value.list)
        parts.push("\nUsage: /omc-switch <workspace-name or email or number>")
      } else if (listResult.ok && listResult.value.totalCount === 1) {
        parts.push(`\nAuto-selected workspace: ${listResult.value.list.split("\n")[1]}`)
      }

      return parts.join("\n\n")
    },
  })
}
