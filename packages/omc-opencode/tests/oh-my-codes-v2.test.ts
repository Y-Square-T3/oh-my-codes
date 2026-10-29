import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { setupV2 } from "../src/plugins/oh-my-codes.js"

const mockModelsResponse = {
  providers: [
    {
      id: "omc-acme",
      name: "Acme AI",
      api: "https://llm.example.com/v1",
      npm: "@ai-sdk/openai-compatible",
      env: [],
      modelCount: 1,
    },
  ],
  models: [
    {
      id: "acme-reasoner",
      providerId: "omc-acme",
      name: "Acme Reasoner",
      family: "acme",
      reasoning: true,
      toolCall: true,
      attachment: true,
      temperature: true,
      openWeights: false,
      modalitiesInput: ["text", "image"],
      modalitiesOutput: ["text"],
      costInput: 3,
      costOutput: 15,
      limitContext: 200000,
      limitOutput: 16384,
      releaseDate: "2025-05-14",
    },
  ],
}

const mockCredentialsResponse = {
  apiKey: "test-oauth-token",
  baseUrl: "https://api.omc.ai/api/v2",
  workspaceId: "ws-123",
}

type FakeEvent = { type: string; data?: Record<string, unknown> }

function stepStarted(overrides: Record<string, unknown> = {}): FakeEvent {
  return {
    type: "session.step.started",
    data: {
      sessionID: "sess-1",
      assistantMessageID: "msg-1",
      agent: "build",
      model: { id: "acme-reasoner", providerID: "omc-acme" },
      ...overrides,
    },
  }
}

function stepEnded(
  finish: string,
  tokens: Record<string, unknown>,
  overrides: Record<string, unknown> = {},
): FakeEvent {
  return {
    type: "session.step.ended",
    data: {
      sessionID: "sess-1",
      assistantMessageID: "msg-1",
      finish,
      tokens,
      cost: 0,
      ...overrides,
    },
  }
}

function mockCtx(events: FakeEvent[] | "infinite") {
  const added: { info: Record<string, unknown>; models: unknown[] }[] = []
  const editor = {
    get: vi.fn().mockReturnValue(undefined),
    add: vi.fn(
      (input: { info: Record<string, unknown>; models: unknown[] }) => {
        added.push(input)
      },
    ),
  }
  let capturedSignal: AbortSignal | undefined

  const ctx = {
    provider: {
      transform: vi.fn(async (cb: (e: typeof editor) => void) => {
        cb(editor)
        return { dispose: async () => {} }
      }),
    },
    event: {
      subscribe: vi.fn((opts?: { types?: string[]; signal?: AbortSignal }) => {
        capturedSignal = opts?.signal
        if (events === "infinite") {
          return {
            [Symbol.asyncIterator]() {
              return {
                next: () =>
                  new Promise(() => {
                    /* never resolves until abort */
                  }),
              }
            },
          }
        }
        return (async function* () {
          for (const e of events) yield e
        })()
      }),
    },
  }

  return {
    ctx,
    editor,
    added,
    getSignal: () => capturedSignal,
  }
}

async function settle() {
  await new Promise((r) => setTimeout(r, 10))
}

let fetchMock: ReturnType<typeof vi.fn>
let consoleLog: ReturnType<typeof vi.spyOn>
let consoleWarn: ReturnType<typeof vi.spyOn>

function tokenCalls() {
  return fetchMock.mock.calls.filter((c) =>
    String(c[0]).includes("/token-usage"),
  ).length
}

function lastTokenBody() {
  const calls = fetchMock.mock.calls.filter((c) =>
    String(c[0]).includes("/token-usage"),
  )
  return JSON.parse(calls[calls.length - 1]![1].body)
}

beforeEach(() => {
  fetchMock = vi.fn((url: string) => {
    if (url.includes("/models")) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockModelsResponse),
      })
    }
    if (url.includes("/account/credentials")) {
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockCredentialsResponse),
      })
    }
    return Promise.resolve({ ok: true })
  })
  vi.stubGlobal("fetch", fetchMock)
  consoleLog = vi.spyOn(console, "log").mockImplementation(() => {})
  consoleWarn = vi.spyOn(console, "warn").mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
  delete process.env.OMC_DAEMON_URL
})

describe("OhMyCodesPlugin (V2 setup)", () => {
  it("should inject providers through ctx.provider.transform", async () => {
    const { ctx, editor, added } = mockCtx([])
    const cleanup = await setupV2(ctx as never)
    await (cleanup ?? (() => {}))?.()

    expect(ctx.provider.transform).toHaveBeenCalledOnce()
    expect(editor.add).toHaveBeenCalledOnce()
    expect(added[0]!.info).toEqual({
      id: "omc-acme",
      name: "Acme AI",
      activation: "enabled",
      package: "aisdk:@ai-sdk/openai-compatible",
      settings: {
        baseURL: "https://api.omc.ai/api/v2",
        apiKey: "test-oauth-token",
      },
      headers: { "x-workspace-id": "ws-123" },
    })

    const [model] = added[0]!.models as Record<string, unknown>[]
    expect(model).toMatchObject({
      id: "acme-reasoner",
      modelID: "acme-reasoner",
      providerID: "omc-acme",
      name: "Acme Reasoner",
      family: "acme",
      capabilities: {
        tools: true,
        input: ["text", "image"],
        output: ["text"],
        temperature: true,
      },
      cost: [{ input: 3, output: 15, cache: { read: 0, write: 0 } }],
      limit: { context: 200000, output: 16384 },
      status: "active",
      enabled: true,
    })
  })

  it("should skip provider IDs already present in the editor", async () => {
    const { ctx, editor, added } = mockCtx([])
    editor.get.mockReturnValue({ provider: {}, models: new Map() })

    const cleanup = await setupV2(ctx as never)
    await (cleanup ?? (() => {}))?.()

    expect(added).toHaveLength(0)
  })

  it("should skip injection when the daemon is unavailable", async () => {
    fetchMock.mockImplementation((url: string) => {
      if (url.includes("/models")) {
        return Promise.resolve({ ok: false, status: 500 })
      }
      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve(mockCredentialsResponse),
      })
    })
    const { ctx, added } = mockCtx([])

    const cleanup = await setupV2(ctx as never)
    await (cleanup ?? (() => {}))?.()

    expect(added).toHaveLength(0)
    expect(ctx.provider.transform).not.toHaveBeenCalled()
    expect(consoleLog).toHaveBeenCalledWith(
      expect.stringContaining("skipping provider injection"),
    )
  })

  it("should send one record per step.ended, each with its own message", async () => {
    const { ctx } = mockCtx([
      stepStarted(),
      stepEnded("tool-calls", {
        input: 10,
        output: 5,
        reasoning: 1,
        cache: { read: 2, write: 3 },
      }),
      stepStarted({
        assistantMessageID: "msg-2",
        model: { id: "acme-reasoner", providerID: "omc-acme" },
      }),
      stepEnded(
        "stop",
        {
          input: 20,
          output: 7,
          reasoning: 2,
          cache: { read: 4, write: 5 },
        },
        { assistantMessageID: "msg-2" },
      ),
    ])

    await setupV2(ctx as never)
    await settle()

    expect(tokenCalls()).toBe(2)
    const bodies = fetchMock.mock.calls
      .filter((c) => String(c[0]).includes("/token-usage"))
      .map((c) => JSON.parse(c[1].body))
    expect(bodies[0]).toMatchObject({
      sessionId: "sess-1",
      messageId: "msg-1",
      agent: "build",
      model: "acme-reasoner",
      inputTokens: 10,
      outputTokens: 5,
      reasoningTokens: 1,
      cacheReadTokens: 2,
      cacheWriteTokens: 3,
    })
    expect(bodies[1]).toMatchObject({
      sessionId: "sess-1",
      messageId: "msg-2",
      inputTokens: 20,
      outputTokens: 7,
    })
  })

  it("should ignore compaction agents", async () => {
    const { ctx } = mockCtx([
      stepStarted({ agent: "compaction" }),
      stepEnded("stop", { input: 100, output: 50 }),
    ])

    await setupV2(ctx as never)
    await settle()

    expect(tokenCalls()).toBe(0)
  })

  it("should dedupe repeated step.ended events for the same message", async () => {
    const { ctx } = mockCtx([
      stepStarted(),
      stepEnded("stop", { input: 1 }),
      stepEnded("stop", { input: 1 }),
    ])

    await setupV2(ctx as never)
    await settle()

    expect(tokenCalls()).toBe(1)
  })

  it("should fall back to unknown model when step.started was missed", async () => {
    const { ctx } = mockCtx([stepEnded("stop", { input: 5, output: 2 })])

    await setupV2(ctx as never)
    await settle()

    expect(lastTokenBody()).toMatchObject({
      agent: "",
      model: "unknown",
      inputTokens: 5,
    })
  })

  it("should use OMC_DAEMON_URL for both injection and token-usage", async () => {
    process.env.OMC_DAEMON_URL = "http://custom:9999"
    const { ctx } = mockCtx([stepStarted(), stepEnded("stop", { input: 1 })])

    await setupV2(ctx as never)
    await settle()

    expect(fetchMock.mock.calls[0]![0]).toBe("http://custom:9999/models")
    const usageCall = fetchMock.mock.calls.find((c) =>
      String(c[0]).includes("/token-usage"),
    )
    expect(usageCall![0]).toBe("http://custom:9999/token-usage")
  })

  it("should abort the event subscription on cleanup", async () => {
    const { ctx, getSignal } = mockCtx("infinite")

    const cleanup = await setupV2(ctx as never)
    expect(ctx.event.subscribe).toHaveBeenCalledOnce()
    expect(getSignal()).toBeInstanceOf(AbortSignal)
    expect(getSignal()!.aborted).toBe(false)

    await (cleanup ?? (() => {}))?.()
    expect(getSignal()!.aborted).toBe(true)
  })

  it("should warn when the event stream throws", async () => {
    const { ctx } = mockCtx([])
    ctx.event.subscribe = vi.fn(() =>
      (async function* () {
        yield stepStarted()
        throw new Error("stream down")
      })(),
    ) as never

    await setupV2(ctx as never)
    await settle()

    expect(consoleWarn).toHaveBeenCalledWith(
      expect.stringContaining("stream down"),
    )
  })

  it("should log daemon failures when sending token-usage", async () => {
    const { ctx } = mockCtx([stepStarted(), stepEnded("stop", { input: 1 })])
    fetchMock.mockImplementation((url: string) => {
      if (url.includes("/models") || url.includes("/account/credentials")) {
        return Promise.resolve({
          ok: true,
          json: () =>
            Promise.resolve(
              url.includes("/models")
                ? mockModelsResponse
                : mockCredentialsResponse,
            ),
        })
      }
      return Promise.resolve({ ok: false, status: 503 })
    })

    await setupV2(ctx as never)
    await settle()

    expect(consoleLog).toHaveBeenCalledWith(
      expect.stringContaining("omcd responded 503"),
    )
  })
})
