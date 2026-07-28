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
if (!key) throw new Error(`Unsupported platform: ${plat}/${arch}`)

const ext = plat === 'win32' ? '.exe' : ''
const platformDir = dirname(require.resolve(`@y-square-t3/oh-my-codes-${key}/package.json`))
const binPath = join(platformDir, 'bin', `omcd${ext}`)

const { status } = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit' })
process.exit(status ?? 0)
