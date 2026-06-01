import { parse, ParseError } from "jsonc-parser"

function stripBom(content: string): string {
  return content.charCodeAt(0) === 0xfeff ? content.slice(1) : content
}

export async function parseJsoncSafe<T = unknown>(content: string): Promise<{ data: T | null; error?: string }> {
  const errors: ParseError[] = []
  const data = parse(stripBom(content), errors, {
    allowTrailingComma: true,
    disallowComments: false,
  }) as T | null

  if (errors.length > 0) {
    return {
      data: null,
      error: `JSONC parse error: ${errors.length} error(s) found`,
    }
  }

  return { data }
}
