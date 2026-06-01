#!/usr/bin/env node
// scripts/generate-changelog.mjs
// Generate changelog for releases using Node.js child_process

import { execSync } from "node:child_process"

const EXCLUDE_PREFIXES = /^(ignore:|test:|chore:|ci:|release:)/i

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

function getCommits(previousTag) {
  try {
    const log = run(`git log ${previousTag}..HEAD --oneline --format="%h %s"`)
    return log
      .split("\n")
      .filter((line) => line && !line.match(EXCLUDE_PREFIXES))
      .map((line) => `- ${line}`)
  } catch {
    return []
  }
}

function main() {
  const previousTag = getLatestReleasedTag()

  if (!previousTag) {
    console.log("Initial release")
    process.exit(0)
  }

  const commits = getCommits(previousTag)

  if (commits.length === 0) {
    console.log("No notable changes")
  } else {
    console.log("## Changelog\n")
    console.log(commits.join("\n"))
  }
}

main()
