#!/usr/bin/env node
import { spawnSync } from 'node:child_process'
import { platform, arch } from 'node:os'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'
import { createRequire } from 'node:module'

const __dirname = dirname(fileURLToPath(import.meta.url))
const require = createRequire(import.meta.url)

const plat = platform()
const archMap = {
  darwin: { arm64: 'darwin-arm64', x64: 'darwin-x64' },
  linux: { arm64: 'linux-arm64', x64: 'linux-x64' },
  win32: { x64: 'win32-x64' },
}
const key = archMap[plat]?.[arch]
if (!key) {
  console.error(`oh-my-codes: unsupported platform ${plat}/${arch}`)
  process.exit(1)
}

const ext = plat === 'win32' ? '.exe' : ''
const pkgName = `@y-square-t3/oh-my-codes-${key}`

let platformDir
try {
  platformDir = dirname(require.resolve(`${pkgName}/package.json`))
} catch {
  console.error(`oh-my-codes: platform binary not found for ${plat}/${arch}`)
  console.error(`Required package: ${pkgName}`)
  console.error(`Try reinstalling: npm install -g oh-my-codes`)
  process.exit(1)
}

const binPath = join(platformDir, 'bin', `omcd${ext}`)

const { status } = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit' })
process.exit(status ?? 0)
