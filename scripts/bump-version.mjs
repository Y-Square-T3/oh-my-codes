#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from "node:fs"
import { execSync } from "node:child_process"
import { fileURLToPath } from "node:url"
import { dirname, join } from "node:path"

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT = join(__dirname, "..")
const OMC_DIR = join(ROOT, "packages/omc")
const MAIN_PKG = join(OMC_DIR, "package.json")
const CARGO_TOML = join(OMC_DIR, "Cargo.toml")

const PLATFORMS = ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-x64"]

const SEMVER_RE = /^\d+\.\d+\.\d+(-[0-9A-Za-z]+(\.[0-9A-Za-z]+)*)?$/

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"))
}

function writeJson(file, data) {
  writeFileSync(file, JSON.stringify(data, null, 2) + "\n")
}

function readMainVersion() {
  return readJson(MAIN_PKG).version
}

function validate(version) {
  if (!SEMVER_RE.test(version)) {
    console.error(`error: Invalid version format: ${version} (expected semver like 1.2.3 or 1.2.3-beta.1)`)
    process.exit(1)
  }
}

function showCurrent() {
  const mainVersion = readMainVersion()
  console.log(`Current version: ${mainVersion}`)
  console.log("")
  console.log("Checking consistency...")

  let inconsistent = false

  for (const platform of PLATFORMS) {
    const pkgFile = join(OMC_DIR, "dist", platform, "package.json")
    if (!existsSync(pkgFile)) {
      console.log(`  ? ${pkgFile} (not found)`)
      continue
    }
    const v = readJson(pkgFile).version
    if (v !== mainVersion) {
      console.log(`  ✗ ${pkgFile}: ${v} (expected ${mainVersion})`)
      inconsistent = true
    } else {
      console.log(`  ✓ ${pkgFile}`)
    }
  }

  const cargoContent = readFileSync(CARGO_TOML, "utf8")
  const match = cargoContent.match(/^version\s*=\s*"(.*)"/m)
  const cargoVersion = match ? match[1] : null
  if (cargoVersion !== mainVersion) {
    console.log(`  ✗ ${CARGO_TOML}: ${cargoVersion} (expected ${mainVersion})`)
    inconsistent = true
  } else {
    console.log(`  ✓ ${CARGO_TOML}`)
  }

  if (inconsistent) {
    console.log("")
    console.log(`Versions are inconsistent. Run: node ${fileURLToPath(import.meta.url)} <version>`)
    process.exit(1)
  }

  console.log("")
  console.log("All versions are consistent.")
}

function bumpVersion(newVersion) {
  validate(newVersion)

  console.log(`Bumping version to ${newVersion}`)
  console.log("")

  console.log(`Updating ${MAIN_PKG} (source of truth)...`)
  const mainPkg = readJson(MAIN_PKG)
  mainPkg.version = newVersion
  writeJson(MAIN_PKG, mainPkg)

  console.log("Updating platform packages...")
  for (const platform of PLATFORMS) {
    const pkgFile = join(OMC_DIR, "dist", platform, "package.json")
    if (!existsSync(pkgFile)) continue
    console.log(`  ${pkgFile}`)
    const pkg = readJson(pkgFile)
    pkg.version = newVersion
    writeJson(pkgFile, pkg)
  }

  console.log(`Updating ${CARGO_TOML}...`)
  let cargoContent = readFileSync(CARGO_TOML, "utf8")
  cargoContent = cargoContent.replace(
    /^(\s*)version\s*=\s*".*"/m,
    `$1version = "${newVersion}"`,
  )
  writeFileSync(CARGO_TOML, cargoContent)

  console.log("Updating Cargo.lock...")
  execSync(`cargo generate-lockfile --manifest-path "${CARGO_TOML}" --quiet`, { stdio: "inherit" })

  console.log("Updating yarn.lock...")
  execSync("yarn install --mode update-lockfile --silent", { cwd: ROOT, stdio: "inherit" })

  console.log("")
  console.log(`✓ Version bumped to ${newVersion}`)
  console.log("")
  console.log("Changed files:")
  console.log(`  - ${MAIN_PKG}`)
  for (const platform of PLATFORMS) {
    console.log(`  - ${join(OMC_DIR, "dist", platform, "package.json")}`)
  }
  console.log(`  - ${CARGO_TOML}`)
  console.log(`  - ${join(OMC_DIR, "Cargo.lock")}`)
  console.log(`  - ${join(ROOT, "yarn.lock")}`)
  console.log("")
  console.log("Next steps:")
  console.log("  1. Review changes: git diff")
  console.log(`  2. Commit: git add -A && git commit -m "release: v${newVersion}"`)
  console.log(`  3. Tag: git tag v${newVersion}`)
  console.log("  4. Push: git push && git push origin v" + newVersion)
}

const version = process.argv[2]
if (version) {
  bumpVersion(version)
} else {
  showCurrent()
}
