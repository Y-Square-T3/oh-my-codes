#!/usr/bin/env bun

import { $ } from "bun"
import { Command } from "commander"
import { confirm, log } from "@clack/prompts"

interface MergeArgs {
  author: string
  source: string
  target: string
  dryRun: boolean
}

function parseArgs(): MergeArgs {
  const program = new Command()
  program
    .name("merge")
    .description("Squash-merge source branch onto target with author rewrite")
    .requiredOption("-a, --author <value>", "Author in 'Name <email>' format")
    .option("-s, --source <branch>", "Source branch", "dev")
    .option("-t, --target <branch>", "Target branch", "internal")
    .option("--dry-run", "Preview changes without applying", false)

  program.parse(process.argv)
  const opts = program.opts()

  return {
    author: opts.author,
    source: opts.source,
    target: opts.target,
    dryRun: opts.dryRun,
  }
}

function validateAuthor(author: string): boolean {
  const emailMatch = author.match(/^[^<]+ <[^>]+>$/)
  return emailMatch !== null
}

async function getCurrentBranch(): Promise<string> {
  const branch = await $`git branch --show-current`.text()
  return branch.trim()
}

async function hasUncommittedChanges(): Promise<boolean> {
  const result = await $`git diff --quiet && git diff --cached --quiet`.nothrow()
  return result.exitCode !== 0
}

async function branchExists(branch: string): Promise<boolean> {
  const result = await $`git rev-parse --verify ${branch}`.nothrow()
  return result.exitCode === 0
}

async function getRemoteBranchSha(branch: string): Promise<string | null> {
  const result = await $`git rev-parse --verify origin/${branch}`.nothrow()
  if (result.exitCode !== 0) return null
  return result.text().trim()
}

async function getLocalBranchSha(branch: string): Promise<string | null> {
  const result = await $`git rev-parse --verify ${branch}`.nothrow()
  if (result.exitCode !== 0) return null
  return result.text().trim()
}

async function getCommitsBetween(from: string, to: string): Promise<number> {
  const result = await $`git log --format=%H ${from}..${to}`.nothrow()
  if (result.exitCode !== 0) return 0
  return result.text().trim().split("\n").filter(Boolean).length
}

async function getLastCommitSha(from: string, to: string): Promise<string> {
  const result = await $`git log --format=%H -1 ${from}..${to}`.text()
  return result.trim()
}

async function getLastMergedSha(branch: string): Promise<string | null> {
  const result = await $`git log -1 --format=%B ${branch}`.text()
  const match = result.match(/feat\(#AI-001\): Merge .* \(([a-f0-9]+)\)/)
  return match ? match[1] : null
}

async function getCurrentAuthor(): Promise<string> {
  const name = await $`git config user.name`.text()
  const email = await $`git config user.email`.text()
  return `${name.trim()} <${email.trim()}>`
}

async function main() {
  const args = parseArgs()

  if (!validateAuthor(args.author)) {
    log.error(`Invalid author format: ${args.author}`)
    log.step("Expected format: 'Name <email>'")
    process.exit(1)
  }

  log.info("=== Merge Dev to Internal ===")
  log.step(`Source branch:  ${args.source}`)
  log.step(`Target branch:  ${args.target}`)
  log.step(`New author:     ${args.author}`)
  log.step(`Dry run:        ${args.dryRun ? "yes" : "no"}`)

  const currentAuthor = await getCurrentAuthor()
  log.step(`Current author: ${currentAuthor}`)

  if (await hasUncommittedChanges()) {
    log.error("Working tree has uncommitted changes. Stash or commit them first.")
    process.exit(1)
  }

  const currentBranch = await getCurrentBranch()
  log.step(`Current branch: ${currentBranch || "(detached)"}`)

  if (!(await branchExists(args.source))) {
    log.error(`Source branch '${args.source}' does not exist`)
    process.exit(1)
  }

  if (!(await branchExists(args.target))) {
    log.error(`Target branch '${args.target}' does not exist`)
    process.exit(1)
  }

  const targetLocalSha = await getLocalBranchSha(args.target)
  const targetRemoteSha = await getRemoteBranchSha(args.target)

  log.step(`\n${args.target} @ ${targetLocalSha}`)
  if (targetRemoteSha && targetRemoteSha !== targetLocalSha) {
    log.step(`origin/${args.target} @ ${targetRemoteSha} (local differs from remote)`)
  }

  const lastMergedSha = await getLastMergedSha(args.target)
  const latestSourceSha = (await $`git rev-parse ${args.source}`.text()).trim().substring(0, 7)

  if (lastMergedSha) {
    log.step("Last merged SHA: " + lastMergedSha)
  }

  const fromRef = lastMergedSha ? lastMergedSha : args.target
  const numCommits = await getCommitsBetween(fromRef, args.source)
  if (numCommits === 0) {
    log.success("No new commits to merge. Branches are already in sync.")
    process.exit(0)
  }
  log.step(
    "Commits to merge: " +
      numCommits +
      " (from " +
      (lastMergedSha || args.target) +
      " to " +
      latestSourceSha.trim() +
      ")",
  )

  if (args.dryRun) {
    log.info("=== DRY RUN ===")
    log.step("Would merge " + numCommits + " commits from " + args.source + " onto " + args.target)
    log.step("Would rewrite all commits with author: " + args.author)
    log.step("Would checkout " + args.target + " and squash-merge")
    process.exit(0)
  }

  const shouldProceed = await confirm({
    message: "Proceed with merge of " + numCommits + " commits?",
    initialValue: true,
  })

  if (shouldProceed === false) {
    log.warn("Aborted.")
    process.exit(0)
  }

  log.step("\nChecking out target branch...")
  await $`git checkout ${args.target}`

  log.step("Merging " + args.source + " onto " + args.target + " with new author...")

  const mergeResult = await $`git merge --squash --no-commit ${args.source}`.nothrow()

  if (mergeResult.exitCode !== 0) {
    log.warn("Conflicts detected. Auto-resolving with theirs (source branch)...")
    await $`git checkout --theirs .`
    await $`git add .`
  }

  const authorName = args.author.split(" ")[0]
  const authorEmail = args.author.split(" ")[1].replace("<", "").replace(">", "")

  const commitMsg = "feat(#AI-001): Merge " + args.source + " -> " + args.target + " (" + latestSourceSha.trim() + ")"
  const commitResult =
    await $`GIT_AUTHOR_NAME="${authorName}" GIT_AUTHOR_EMAIL="${authorEmail}" GIT_COMMITTER_NAME="${authorName}" GIT_COMMITTER_EMAIL="${authorEmail}" git commit -m "${commitMsg}"`.nothrow()

  if (commitResult.exitCode !== 0) {
    log.error("Merge failed.")
    log.step('Re-run: bun run script/merge.ts -a "' + args.author + '" -s ' + args.source + " -t " + args.target)
    process.exit(1)
  }

  log.success("Merge completed successfully!")

  const shouldPush = await confirm({
    message: "Push to remote?",
    initialValue: true,
  })

  if (shouldPush === false) {
    log.warn("Push skipped. Run 'git push' to push manually.")
    return
  }

  log.step(`Pushing ${args.target}...`)
  const pushResult = await $`git push`.nothrow()

  if (pushResult.exitCode !== 0) {
    log.error("Push failed:")
    log.error(pushResult.stderr.toString())
    process.exit(1)
  }

  log.step("\nChecking out source branch...")
  await $`git checkout ${args.source}`

  log.success("Done!")
}

main().catch((error) => {
  log.error(`Fatal error: ${error}`)
  process.exit(1)
})
