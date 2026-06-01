#!/usr/bin/env node
// scripts/build-binaries.mjs
// Build platform-specific binaries for CLI distribution using @yao-pkg/pkg

import { execSync } from "node:child_process"
import { existsSync, mkdirSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = join(__dirname, "..")

const PLATFORMS = [
  {
    dir: "darwin-arm64",
    target: "node24-macos-arm64",
    binary: "oh-my-codes",
    description: "macOS ARM64",
  },
  {
    dir: "darwin-x64",
    target: "node24-macos-x64",
    binary: "oh-my-codes",
    description: "macOS x64",
  },
  {
    dir: "linux-x64",
    target: "node24-linux-x64",
    binary: "oh-my-codes",
    description: "Linux x64 (glibc)",
  },
  {
    dir: "windows-x64",
    target: "node24-win-x64",
    binary: "oh-my-codes.exe",
    description: "Windows x64",
  },
]

const ENTRY_POINT = "src/cli/index.ts"
const BUNDLED_ENTRY = "dist/cli/bundle-for-pkg.js"

function run(command, options = {}) {
  const cwd = options.cwd || rootDir
  try {
    return execSync(command, { cwd, stdio: "inherit" })
  } catch (error) {
    if (!options.ignoreError) {
      throw error
    }
  }
}

async function buildBundle() {
  console.log("\n[1/2] Bundling TypeScript entry point...")

  mkdirSync(join(rootDir, "dist", "cli"), { recursive: true })

  const cmd = `npx esbuild ${ENTRY_POINT} --bundle --platform=node --outfile=${BUNDLED_ENTRY} --format=cjs --external:@ast-grep/napi`
  run(cmd)

  if (!existsSync(join(rootDir, BUNDLED_ENTRY))) {
    throw new Error(`Bundle not found at ${BUNDLED_ENTRY}`)
  }

  console.log("   Bundle created successfully")
}

async function buildPlatform(platform) {
  const outfile = join("packages", platform.dir, "bin", platform.binary)
  const absOutfile = join(rootDir, outfile)

  console.log(`\n[2/2] Building ${platform.description}...`)
  console.log(`   Target: ${platform.target}`)
  console.log(`   Output: ${outfile}`)

  try {
    mkdirSync(join(rootDir, "packages", platform.dir, "bin"), { recursive: true })

    const cmd = `npx pkg ${BUNDLED_ENTRY} -c ${join(rootDir, "package.json")} --target ${platform.target} --output ${absOutfile} --compress GZip`
    run(cmd)

    if (!existsSync(absOutfile)) {
      console.error(`   Binary not found after build: ${outfile}`)
      return false
    }

    if (process.platform !== "win32") {
      try {
        const fileInfo = execSync(`file "${absOutfile}"`, { encoding: "utf8" }).trim()
        console.log(`   ${fileInfo}`)
      } catch {
        console.log(`   Binary created successfully`)
      }
    } else {
      console.log(`   Binary created successfully`)
    }

    return true
  } catch (error) {
    console.error(`   Build failed: ${error.message}`)
    return false
  }
}

async function main() {
  console.log("Building oh-my-codes platform binaries")
  console.log(`Entry point: ${ENTRY_POINT}`)
  console.log(`Platforms: ${PLATFORMS.length}`)

  if (!existsSync(join(rootDir, ENTRY_POINT))) {
    console.error(`Entry point not found: ${ENTRY_POINT}`)
    process.exit(1)
  }

  await buildBundle()

  const results = []
  for (const platform of PLATFORMS) {
    const success = await buildPlatform(platform)
    results.push({ platform: platform.description, success })
  }

  console.log("\n" + "=".repeat(50))
  console.log("Build Summary:")
  console.log("=".repeat(50))

  const succeeded = results.filter((r) => r.success).length
  const failed = results.filter((r) => !r.success).length

  for (const result of results) {
    const icon = result.success ? "[OK]" : "[FAIL]"
    console.log(`  ${icon} ${result.platform}`)
  }

  console.log("=".repeat(50))
  console.log(`Total: ${succeeded} succeeded, ${failed} failed`)

  if (failed > 0) {
    process.exit(1)
  }

  console.log("\nAll platform binaries built successfully!\n")
}

main().catch((error) => {
  console.error("Fatal error:", error)
  process.exit(1)
})
