function stripJsonCommentsAndTrailingCommas(content: string): string {
  let result = ""
  let inString = false
  let stringChar = ""
  let i = 0

  while (i < content.length) {
    // Handle strings
    if (inString) {
      result += content[i]
      if (content[i] === stringChar && content[i - 1] !== "\\") {
        inString = false
      }
      i++
      continue
    }

    // Enter string
    if (content[i] === '"' || content[i] === "'") {
      inString = true
      stringChar = content[i]
      result += content[i]
      i++
      continue
    }

    // Remove single-line comments
    if (content[i] === "/" && content[i + 1] === "/") {
      while (i < content.length && content[i] !== "\n") {
        i++
      }
      continue
    }

    // Remove multi-line comments
    if (content[i] === "/" && content[i + 1] === "*") {
      i += 2
      while (i < content.length - 1 && !(content[i] === "*" && content[i + 1] === "/")) {
        i++
      }
      i += 2
      continue
    }

    result += content[i]
    i++
  }

  // Remove trailing commas before } or ] (handles arrays and objects)
  result = result.replace(/,\s*([}\]])/g, "$1")

  return result
}

function stripBom(content: string): string {
  return content.charCodeAt(0) === 0xfeff ? content.slice(1) : content
}

export async function parseJsoncSafe<T = unknown>(
  content: string,
): Promise<{ data: T | null; error?: string }> {
  try {
    const cleaned = stripBom(stripJsonCommentsAndTrailingCommas(content))
    const data = JSON.parse(cleaned) as T
    return { data }
  } catch (e) {
    const error = e instanceof Error ? e.message : String(e)
    return { data: null, error }
  }
}
