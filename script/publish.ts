#!/usr/bin/env bun

import { Command } from "commander"
import { $ } from "bun"
import { existsSync } from "node:fs"
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

async function renamePackageJson(
  pkgPath: string,
  oldName: string,
  newName: string,
): Promise<void> {
  const content = await Bun.file(pkgPath).text()
  const updated = content.replace(new RegExp(`"${oldName}"`, "g"), `"${newName}"`)
  await Bun.write(pkgPath, updated)
}

async function renameAllPackages(
  baseName: string,
  newBaseName: string,
): Promise<void> {
  const mainPkgPath = new URL("../package.json", import.meta.url).pathname
  const mainContent = await Bun.file(mainPkgPath).text()
  const pkgJson = JSON.parse(mainContent)
  const currentVersion = pkgJson.version

  console.log(`Renaming packages to ${newBaseName}@${currentVersion}...`)

  for (const platform of PLATFORM_PACKAGES) {
    const oldPlatformName = `${baseName}-${platform}`
    const newPlatformName = `${newBaseName}-${platform}`
    const pkgPath = new URL(`../packages/${platform}/package.json`, import.meta.url).pathname

    if (existsSync(pkgPath)) {
      await renamePackageJson(pkgPath, oldPlatformName, newPlatformName)
      console.log(`  Renamed: packages/${platform}`)
    } else {
      console.warn(`  Warning: packages/${platform}/package.json not found`)
    }
  }

  await renamePackageJson(mainPkgPath, baseName, newBaseName)
  console.log(`  Renamed: package.json`)

  for (const platform of PLATFORM_PACKAGES) {
    const oldPlatformName = `${baseName}-${platform}`
    const newPlatformName = `${newBaseName}-${platform}`
    await renamePackageJson(mainPkgPath, oldPlatformName, newPlatformName)
  }
  console.log(`  Renamed: optionalDependencies in package.json`)
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
  baseName: string,
  newBaseName: string,
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
        const pkgName = `${newBaseName}-${platform}`

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
  const mainResult = await publishPackage(process.cwd(), distTag, true, newBaseName, version)

  if (mainResult.success) {
    if (mainResult.alreadyPublished) {
      console.log(`  ✓ ${newBaseName}@${version} (already published)`)
    } else {
      console.log(`  ✓ ${newBaseName}@${version}`)
    }
  } else {
    console.error(`  ✗ ${newBaseName} failed: ${mainResult.error}`)
    throw new Error(`Failed to publish ${newBaseName}`)
  }
}

async function main() {
  const program = new Command()

  program
    .name("publish")
    .description("Publish oh-my-codes packages with custom naming")
    .requiredOption("--name <name>", "Base package name (e.g., @myorg/my-matrix)")
    .option("--dist-tag <tag>", "npm dist-tag (auto-detected from version if not specified)")
    .option("--skip-platform", "Skip publishing platform packages")

  program.parse(process.argv)

  const opts = program.opts()
  const newBaseName = opts.name
  const distTagOverride = opts.distTag
  const skipPlatform = opts.skipPlatform ?? false

  const baseName = "oh-my-matrix"

  const mainPkgPath = new URL("../package.json", import.meta.url).pathname
  const mainContent = await Bun.file(mainPkgPath).text()
  const pkgJson = JSON.parse(mainContent)
  const version = pkgJson.version

  console.log(`=== Publishing oh-my-codes (rename to ${newBaseName}) ===\n`)
  console.log(`Version: ${version}`)
  if (distTagOverride) {
    console.log(`Dist-tag: ${distTagOverride}`)
  }

  const distTag = distTagOverride ?? getDistTag(version)

  await renameAllPackages(baseName, newBaseName)
  await publishAllPackages(baseName, newBaseName, version, distTag, skipPlatform)

  console.log(`\n=== Successfully published ${newBaseName}@${version} ===`)
}

main()