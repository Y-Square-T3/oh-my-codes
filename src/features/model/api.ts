import { normalizeServerUrl } from "../account/url"
import { type AccountID } from "../account/schema"
import { Provider } from "./type"

export class ModelApiError extends Error {
  readonly _tag = "ModelApiError"
  constructor(
    readonly method: string,
    readonly url: string,
    readonly description?: string,
  ) {
    super(`ModelApiError: ${method} ${url}${description ? ` - ${description}` : ""}`)
    this.name = "ModelApiError"
  }
}

export async function fetchApiJson(
  accountUrl: string,
  accountId: AccountID,
  accessToken: string,
): Promise<Record<string, Provider>> {
  const normalized = normalizeServerUrl(accountUrl)
  const url = `${normalized}/models/api.json`

  const response = await fetch(url, {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      Accept: "application/json",
    },
  })

  if (!response.ok) {
    const text = await response.text().catch(() => "")
    throw new ModelApiError("GET", url, `HTTP ${response.status}: ${text.slice(0, 200)}`)
  }

  const json = (await response.json()) as unknown

  const parsed = Provider.array().safeParse(Object.values(json as Record<string, Provider>))
  if (!parsed.success) {
    throw new ModelApiError("GET", url, `Parse error: ${parsed.error.message}`)
  }

  const result: Record<string, Provider> = {}
  for (const provider of parsed.data) {
    result[provider.id] = provider
  }

  return result
}
