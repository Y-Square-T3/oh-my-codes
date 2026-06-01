import { copyFileSync, existsSync } from "node:fs"

export interface BackupResult {
  success: boolean
  backupPath?: string
  error?: string
}

export function backupConfigFile(configPath: string): BackupResult {
  if (!existsSync(configPath)) {
    return { success: true }
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, "-")
  const backupPath = `${configPath}.backup-${timestamp}`

  try {
    copyFileSync(configPath, backupPath)
    return { success: true, backupPath }
  } catch (err) {
    return {
      success: false,
      error: err instanceof Error ? err.message : "Failed to create backup",
    }
  }
}
