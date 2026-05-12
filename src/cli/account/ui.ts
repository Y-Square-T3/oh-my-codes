import { Effect, Option } from "effect"
import * as p from "@clack/prompts"
import pc from "picocolors"

import type { AccountWorkspace, AccountInfo, Workspace, WorkspaceID, AccountID } from "../../features/account"

export const intro = (title: string): Effect.Effect<void> =>
  Effect.sync(() => p.intro(pc.cyan(title)))

export const outro = (message: string, exitCode = 0): Effect.Effect<void> =>
  Effect.sync(() => {
    if (exitCode === 0) {
      p.outro(pc.green(message))
    } else {
      p.cancel(message)
    }
  })

export const logInfo = (message: string): Effect.Effect<void> =>
  Effect.sync(() => p.note(message, pc.blue("info")))

export const logSuccess = (message: string): Effect.Effect<void> =>
  Effect.sync(() => p.note(message, pc.green("success")))

export const logError = (message: string): Effect.Effect<void> =>
  Effect.sync(() => p.note(message, pc.red("error")))

export const createSpinner = (): {
  start: (msg: string) => Effect.Effect<void>
  stop: (msg: string, exitCode?: number) => Effect.Effect<void>
} => {
  const spinner = p.spinner()
  return {
    start: (msg: string) => Effect.sync(() => spinner.start(msg)),
    stop: (msg: string, exitCode = 0) =>
      Effect.sync(() => {
        if (exitCode === 0) {
          spinner.stop(msg)
        } else {
          spinner.stop(msg, exitCode)
        }
      }),
  }
}

export const println = (msg: string): Effect.Effect<void> =>
  Effect.sync(() => console.log(msg))

export const dim = (value: string): string => pc.dim(value)

export const activeSuffix = (isActive: boolean): string =>
  isActive ? dim(" (active)") : ""

export const formatAccountLabel = (account: Pick<AccountInfo, "email" | "url">, isActive: boolean): string =>
  `${account.email} ${dim(account.url)}${activeSuffix(isActive)}`

export const formatWorkspaceChoiceLabel = (
  account: Pick<AccountInfo, "email">,
  workspace: Pick<Workspace, "name">,
  isActive: boolean,
): string => `${workspace.name} (${account.email})${activeSuffix(isActive)}`

export const formatOrgLine = (
  account: Pick<AccountInfo, "email" | "url">,
  workspace: { id: string; name: string },
  isActive: boolean,
): string => {
  const dot = isActive ? pc.green("●") : " "
  const name = isActive ? pc.bold(pc.cyan(workspace.name)) : workspace.name
  return `  ${dot} ${name}  ${dim(account.email)}  ${dim(account.url)}  ${dim(workspace.id)}`
}

export const isActiveWorkspaceChoice = (
  active: Option.Option<{ id: AccountID; active_workspace_id: WorkspaceID | null }>,
  choice: { accountID: AccountID; workspaceID: WorkspaceID },
): boolean =>
  Option.isSome(active) &&
  active.value.id === choice.accountID &&
  active.value.active_workspace_id === choice.workspaceID

interface WorkspaceChoice {
  workspaceID: WorkspaceID
  accountID: AccountID
  label: string
}

export const selectWorkspaceEffect = (
  groups: AccountWorkspace[],
  active: Option.Option<{ id: AccountID; active_workspace_id: WorkspaceID | null }>,
): Effect.Effect<Option.Option<WorkspaceChoice>> =>
  Effect.gen(function* () {
    if (groups.length === 0) {
      yield* outro("No workspaces found")
      return Option.none()
    }

    const opts: { value: WorkspaceChoice; label: string }[] = groups.flatMap((group) =>
      group.workspaces.map((workspace) => {
        const isActive = isActiveWorkspaceChoice(active, {
          accountID: group.account.id,
          workspaceID: workspace.id,
        })
        return {
          value: {
            workspaceID: workspace.id,
            accountID: group.account.id,
            label: workspace.name,
          },
          label: formatWorkspaceChoiceLabel(group.account, workspace, isActive),
        }
      }),
    )

    if (opts.length === 0) {
      yield* outro("No workspaces found")
      return Option.none()
    }

    const selected = yield* select(opts, "Select workspace")
    if (Option.isSome(selected)) {
      return Option.some(selected.value)
    }

    yield* outro("Done")
    return Option.none()
  })

import type { Option as ClackOption } from "@clack/prompts"

export const select = <T>(
  options: ClackOption<T>[],
  message: string,
): Effect.Effect<Option.Option<T>> =>
  Effect.tryPromise(() =>
    p.select({
      message,
      options,
    }).then((choice) => {
      if (p.isCancel(choice)) {
        return Option.none()
      }
      return Option.some(choice as T)
    }),
  ).pipe(Effect.orDie)

export const openBrowser = (url: string): Effect.Effect<void, never, never> =>
  Effect.sync(() => {
    if (process.platform === "win32") {
      void Bun.spawn(["cmd", "/c", "start", "", url])
    } else if (process.platform === "darwin") {
      void Bun.spawn(["open", url])
    } else {
      void Bun.spawn(["xdg-open", url])
    }
  })
