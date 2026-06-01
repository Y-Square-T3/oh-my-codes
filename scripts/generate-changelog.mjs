#!/usr/bin/env node
// scripts/generate-changelog.mjs
// Generate changelog for releases using Node.js child_process

import { execSync } from "node:child_process"

const TEAM = ["actions-user", "github-actions[bot]", "code-yeongyu"]

function run(command) {
  try {
    return execSync(command, { encoding: "utf8" }).trim()
  } catch {
    return ""
  }
}

function getLatestReleasedTag() {
  try {
    const output = run(
      `gh release list --exclude-drafts --exclude-pre-releases --limit 1 --json tagName --jq '.[0].tagName // empty'`,
    )
    return output || null
  } catch {
    return null
  }
}

function generateChangelog(previousTag) {
  const notes = []
  try {
    const log = run(`git log ${previousTag}..HEAD --oneline --format="%h %s"`)
    const commits = log
      .split("\n")
      .filter((line) => line && !line.match(/^\w+ (ignore:|test:|chore:|ci:|release:)/i))

    if (commits.length > 0) {
      for (const commit of commits) {
        notes.push(`- ${commit}`)
      }
    }
  } catch {
    // No previous tags found
  }
  return notes
}

function getChangedFiles(previousTag) {
  try {
    const diff = run(`git diff --name-only ${previousTag}..HEAD`)
    return diff
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
  } catch {
    return []
  }
}

function touchesAnyPath(files, candidates) {
  return files.some((file) => candidates.some((candidate) => file === candidate || file.startsWith(`${candidate}/`)))
}

function buildReleaseFraming(files) {
  const bullets = []

  if (touchesAnyPath(files, ["src/index.ts", "bin/platform.js", "postinstall.mjs"])) {
    bullets.push(
      "Rename transition updates across package detection, plugin/config compatibility, and install surfaces.",
    )
  }

  if (touchesAnyPath(files, [".github/workflows", "postinstall.mjs"])) {
    bullets.push(
      "Install and publish workflow hardening, including safer release sequencing and package/install fixes.",
    )
  }

  if (bullets.length === 0) {
    return []
  }

  return [
    "## Minor Compatibility and Stability Release",
    "",
    "This release carries compatibility-facing behavior changes and operational hardening. Read the summary below before upgrading or publishing.",
    "",
    ...bullets.map((bullet) => `- ${bullet}`),
    "",
    "## Commit Summary",
    "",
  ]
}

function getContributors(previousTag) {
  const notes = []
  try {
    const compare = run(
      `gh api "/repos/Y-Square-T3/oh-my-codes/compare/${previousTag}...HEAD" --jq '.commits[] | {login: .author.login, message: .commit.message}'`,
    )
    const contributors = new Map()

    for (const line of compare.split("\n").filter(Boolean)) {
      const { login, message } = JSON.parse(line)
      const title = message.split("\n")[0] ?? ""
      if (title.match(/^(ignore:|test:|chore:|ci:|release:)/i)) continue

      if (login && !TEAM.includes(login)) {
        if (!contributors.has(login)) contributors.set(login, [])
        contributors.get(login)?.push(title)
      }
    }

    if (contributors.size > 0) {
      notes.push("")
      notes.push(`**Thank you to ${contributors.size} community contributor${contributors.size > 1 ? "s" : ""}:**`)
      for (const [username, userCommits] of contributors) {
        notes.push(`- @${username}:`)
        for (const commit of userCommits) {
          notes.push(`  - ${commit}`)
        }
      }
    }
  } catch {
    // Failed to fetch contributors
  }
  return notes
}

function main() {
  const previousTag = getLatestReleasedTag()

  if (!previousTag) {
    console.log("Initial release")
    process.exit(0)
  }

  const changedFiles = getChangedFiles(previousTag)
  const changelog = generateChangelog(previousTag)
  const contributors = getContributors(previousTag)
  const framing = buildReleaseFraming(changedFiles)
  const notes = [...framing, ...changelog, ...contributors]

  if (notes.length === 0) {
    console.log("No notable changes")
  } else {
    console.log(notes.join("\n"))
  }
}

main()
