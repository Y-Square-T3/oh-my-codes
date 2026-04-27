#!/usr/bin/env bun

import { Command } from "commander"
import { $ } from "bun"
import { join } from "node:path"

const PLATFORM_PACKAGES = [
  "darwin-arm64",
  "darwin-x64",
  "darwin-x64-baseline",
  "linux-arm64",
  "linux-arm64-musl",
  "linux-x64",
  "linux-x64-baseline",
  "linux-x64-musl",
  "linux-x64-musl-baseline",
  "windows-x64",
  "windows-x64-baseline",
]

const PACKAGE_NAME = "oh-my-codes"

interface PublishResult {
  success: boolean
  alreadyPublished?: boolean
  error?: string
}

async function checkPackageVersionExists(pkgName: string, version: string): Promise<boolean> {
  try {
    const res = await fetch(`https://registry.npmjs.org/${pkgName}/${version}`)
    return res.ok
  } catch {
    return false
  }
}

function getDistTag(version: string): string | null {
  if (!version.includes("-")) return null
  const prerelease = version.split("-")[1]
  const tag = prerelease?.split(".")[0]
  return tag || "next"
}

async function publishPackage(
  cwd: string,
  distTag: string | null,
  useProvenance = true,
  pkgName?: string,
  version?: string,
): Promise<PublishResult> {
  const tagArgs = distTag ? ["--tag", distTag] : []
  const provenanceArgs = process.env.CI && useProvenance ? ["--provenance"] : []
  const env = useProvenance ? {} : { NPM_CONFIG_PROVENANCE: "false" }

  try {
    await $`npm publish --access public --ignore-scripts ${provenanceArgs} ${tagArgs}`
      .cwd(cwd)
      .env({ ...process.env, ...env })
    return { success: true }
  } catch (error: any) {
    const stderr = error?.stderr?.toString() || error?.message || ""

    if (
      stderr.includes("EPUBLISHCONFLICT") ||
      stderr.includes("E409") ||
      stderr.includes("cannot publish over") ||
      stderr.includes("You cannot publish over the previously published versions")
    ) {
      return { success: true, alreadyPublished: true }
    }

    if (stderr.includes("E403")) {
      if (pkgName && version) {
        const exists = await checkPackageVersionExists(pkgName, version)
        if (exists) {
          return { success: true, alreadyPublished: true }
        }
      }
      return { success: false, error: stderr }
    }

    return { success: false, error: stderr }
  }
}

async function publishAllPackages(
  version: string,
  distTag: string | null,
  skipPlatform: boolean,
): Promise<void> {
  if (skipPlatform) {
    console.log("\nSkipping platform packages (--skip-platform)")
  } else {
    console.log("\nPublishing platform packages in batches...")

    const BATCH_SIZE = 2
    const failures: string[] = []

    for (let i = 0; i < PLATFORM_PACKAGES.length; i += BATCH_SIZE) {
      const batch = PLATFORM_PACKAGES.slice(i, i + BATCH_SIZE)
      const batchNum = Math.floor(i / BATCH_SIZE) + 1
      const totalBatches = Math.ceil(PLATFORM_PACKAGES.length / BATCH_SIZE)

      console.log(`\n  Batch ${batchNum}/${totalBatches}: ${batch.join(", ")}`)

      const publishPromises = batch.map(async (platform) => {
        const pkgDir = join(process.cwd(), "packages", platform)
        const pkgName = `${PACKAGE_NAME}-${platform}`

        console.log(`    Starting ${pkgName}...`)
        const result = await publishPackage(pkgDir, distTag, false, pkgName, version)

        return { platform, pkgName, result }
      })

      const results = await Promise.all(publishPromises)

      for (const { pkgName, result } of results) {
        if (result.success) {
          if (result.alreadyPublished) {
            console.log(`    ✓ ${pkgName}@${version} (already published)`)
          } else {
            console.log(`    ✓ ${pkgName}@${version}`)
          }
        } else {
          console.error(`    ✗ ${pkgName} failed: ${result.error}`)
          failures.push(pkgName)
        }
      }
    }

    if (failures.length > 0) {
      throw new Error(`Failed to publish: ${failures.join(", ")}`)
    }
  }

  console.log(`\nPublishing main package...`)
  const mainResult = await publishPackage(process.cwd(), distTag, true, PACKAGE_NAME, version)

  if (mainResult.success) {
    if (mainResult.alreadyPublished) {
      console.log(`  ✓ ${PACKAGE_NAME}@${version} (already published)`)
    } else {
      console.log(`  ✓ ${PACKAGE_NAME}@${version}`)
    }
  } else {
    console.error(`  ✗ ${PACKAGE_NAME} failed: ${mainResult.error}`)
    throw new Error(`Failed to publish ${PACKAGE_NAME}`)
  }
}

async function main() {
  const program = new Command()

  program
    .name("publish")
    .description("Publish oh-my-codes packages")
    .option("--dist-tag <tag>", "npm dist-tag (auto-detected from version if not specified)")
    .option("--skip-platform", "Skip publishing platform packages")

  program.parse(process.argv)

  const opts = program.opts()
  const distTagOverride = opts.distTag
  const skipPlatform = opts.skipPlatform ?? false

  const mainPkgPath = new URL("../package.json", import.meta.url).pathname
  const mainContent = await Bun.file(mainPkgPath).text()
  const pkgJson = JSON.parse(mainContent)
  const version = pkgJson.version

  console.log(`=== Publishing ${PACKAGE_NAME} ===\n`)
  console.log(`Version: ${version}`)
  if (distTagOverride) {
    console.log(`Dist-tag: ${distTagOverride}`)
  }

  const distTag = distTagOverride ?? getDistTag(version)

  await publishAllPackages(version, distTag, skipPlatform)

  console.log(`\n=== Successfully published ${PACKAGE_NAME}@${version} ===`)
}

main()