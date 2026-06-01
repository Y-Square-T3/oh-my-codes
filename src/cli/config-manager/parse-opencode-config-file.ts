import { readFileSync, statSync } from "node:fs"
import { parseJsoncSafe } from "./jsonc-parser"

export interface OpenCodeConfig {
  plugin?: string[]
  [key: string]: unknown
}

interface ParseConfigResult {
  config: OpenCodeConfig | null
  error?: string
}

function isEmptyOrWhitespace(content: string): boolean {
  return content.trim().length === 0
}

export async function parseOpenCodeConfigFileWithError(path: string): Promise<ParseConfigResult> {
  try {
    const stat = statSync(path)
    if (stat.size === 0) {
      return {
        config: null,
        error: `Config file is empty: ${path}. Delete it or add valid JSON content.`,
      }
    }

    const content = readFileSync(path, "utf-8")
    if (isEmptyOrWhitespace(content)) {
      return {
        config: null,
        error: `Config file contains only whitespace: ${path}. Delete it or add valid JSON content.`,
      }
    }

    const parseResult = await parseJsoncSafe<OpenCodeConfig>(content)

    if (parseResult.error) {
      return {
        config: null,
        error: `${parseResult.error}: ${path}. Check for missing commas, brackets, or invalid characters.`,
      }
    }

    if (parseResult.data == null) {
      return {
        config: null,
        error: `Config file parsed to null/undefined: ${path}. Ensure it contains valid JSON.`,
      }
    }

    if (typeof parseResult.data !== "object" || Array.isArray(parseResult.data)) {
      return {
        config: null,
        error: `Config file must contain a JSON object, not ${Array.isArray(parseResult.data) ? "an array" : typeof parseResult.data}: ${path}`,
      }
    }

    return { config: parseResult.data }
  } catch (err) {
    return {
      config: null,
      error: err instanceof Error ? err.message : `Failed to parse config file: ${path}`,
    }
  }
}
