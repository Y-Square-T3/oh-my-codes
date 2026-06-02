import { execFile } from "child_process"
import { Cause, Duration, Effect, Option } from "effect"
import { tool, type ToolDefinition } from "@opencode-ai/plugin"
import type { PluginContext } from "../types"
import * as Account from "../features/account"
import { PollResult, PollSuccess, Login } from "../features/account/schema"
import { log } from "../features/log/logger"
import { type Result, showToast, runRefreshModels } from "./shared"

type LoginToolArgs = {
  url?: string
}

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

async function runGetWorkspaces(): Promise<Result<{ list: string; totalCount: number; firstWorkspaceName: string | null }>> {
  const exit = await Effect.runPromiseExit(
    Effect.gen(function* () {
      const accountSvc = yield* Account.Service
      const groups = yield* accountSvc.workspacesByAccount()

      const active = yield* accountSvc.active().pipe(Effect.catchAll(() => Effect.succeed(Option.none())))
      const activeAccountId = Option.map(active, (a) => a.id)

      const lines: string[] = []
      let count = 0
      let firstName: string | null = null

      for (const group of groups) {
        for (const ws of group.workspaces) {
          count++
          if (firstName === null) firstName = ws.name
          const isActive =
            Option.isSome(activeAccountId) &&
            activeAccountId.value === group.account.id &&
            group.account.activeWorkspaceId === ws.id
          const marker = isActive ? " (active)" : ""
          lines.push(`${count}. ${ws.name} (${group.account.email}, ${group.account.url})${marker}`)
        }
      }

      if (count === 0) {
        return { list: "No workspaces found.", totalCount: 0, firstWorkspaceName: null }
      }

      return {
        list: `Available workspaces:\n${lines.join("\n")}`,
        totalCount: count,
        firstWorkspaceName: firstName,
      }
    }).pipe(Effect.provide(Account.defaultLayer)),
  )

  if (exit._tag === "Success") {
    return { ok: true, value: exit.value }
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

export function createLoginTool(ctx: PluginContext): ToolDefinition {
  return tool({
    description:
      "Login to an OMC (oh-my-codes) account using device code flow. If no URL is provided, ask the user for their OMC server URL first. This will open a browser for authentication.",
    args: {
      url: tool.schema
        .string()
        .optional()
        .describe("The OMC server URL (e.g. https://server.omc.ai). If not provided, ask the user for it."),
    },
    async execute(args: LoginToolArgs): Promise<string> {
      const serverUrl = args.url
      if (!serverUrl || serverUrl.trim() === "") {
        return "MISSING_URL: Please ask the user for their OMC server URL before proceeding with login."
      }

      await showToast(ctx, "Starting OMC login...", "warning")

      const loginInitResult = await runInitLogin(serverUrl)
      if (!loginInitResult.ok) {
        await showToast(ctx, "Login failed", "error")
        return `LOGIN_FAILED: ${loginInitResult.error}`
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
        return `AUTH_FAILED: ${pollResult.error}`
      }

      const email = pollResult.value
      await showToast(ctx, `Logged in as ${email}`, "success")

      const workspaceResult = await runGetWorkspaces()

      if (workspaceResult.ok && workspaceResult.value.totalCount === 1 && workspaceResult.value.firstWorkspaceName) {
        await runSelectFirstWorkspace()
        await runRefreshModels()
        return `LOGIN_SUCCESS: Successfully logged in as ${email}.\nWorkspace auto-selected: ${workspaceResult.value.firstWorkspaceName}.\n\nThe user is now ready to use OMC.`
      }

      await runRefreshModels()

      if (!workspaceResult.ok) {
        log("[omc-login] Failed to get workspace list", { error: workspaceResult.error })
        return `LOGIN_SUCCESS: Successfully logged in as ${email}.\n\nHowever, failed to retrieve workspaces: ${workspaceResult.error}\nPlease ask the user to run /omc-switch to select a workspace.`
      }

      if (workspaceResult.value.totalCount === 0) {
        return `LOGIN_SUCCESS: Successfully logged in as ${email}.\n\nNo workspaces found for this account.`
      }

      return `LOGIN_SUCCESS: Successfully logged in as ${email}.\n\n${workspaceResult.value.list}\n\nPlease ask the user which workspace they would like to use, then call the omc-switch tool with their selection.`
    },
  })
}
