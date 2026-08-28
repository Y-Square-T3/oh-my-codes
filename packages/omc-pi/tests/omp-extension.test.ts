import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import type {
  ExtensionAPI,
  ExtensionContext,
  MessageEndEvent,
} from "../src/types.js"
import extension from "../src/index.js"

function mockPi() {
  type EventHandler = (event: unknown, ctx: unknown) => void
  const handlers = new Map<string, EventHandler[]>()
  return {
    on: vi.fn((event: string, handler: EventHandler) => {
      if (!handlers.has(event)) handlers.set(event, [])
      handlers.get(event)!.push(handler)
    }),
    logger: {
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
      debug: vi.fn(),
    },
    _emit: (event: string, data: unknown, ctx: unknown) => {
      const fns = handlers.get(event) ?? []
      for (const fn of fns) fn(data, ctx)
    },
    _handlers: handlers,
  } as unknown as ExtensionAPI & {
    _emit: (event: string, data: unknown, ctx: unknown) => void
    _handlers: Map<string, EventHandler[]>
  }
}

function mockCtx(sessionId = "test-session-id") {
  return {
    sessionManager: {
      getSessionId: () => sessionId,
    },
  } as unknown as ExtensionContext
}

function mockAssistantMessage(overrides = {}) {
  return {
    role: "assistant",
    model: "claude-sonnet-4-20250514",
    usage: {
      input: 100,
      output: 50,
      cacheRead: 20,
      cacheWrite: 10,
    },
    id: "msg-123",
    ...overrides,
  }
}

describe("omp.sh extension", () => {
  let fetchMock: ReturnType<typeof vi.fn>
  let pi: ReturnType<typeof mockPi>

  beforeEach(() => {
    fetchMock = vi.fn().mockResolvedValue({ ok: true })
    vi.stubGlobal("fetch", fetchMock)
    pi = mockPi()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    delete process.env.OMC_DAEMON_URL
  })

  it("should export a default function", () => {
    expect(typeof extension).toBe("function")
  })

  it("should register message_end handler", () => {
    extension(pi)
    expect(pi.on).toHaveBeenCalledWith("message_end", expect.any(Function))
  })

  it("should send token usage to daemon on message_end", () => {
    extension(pi)

    const message = mockAssistantMessage()
    const ctx = mockCtx("session-abc")

    pi._emit(
      "message_end",
      { type: "message_end", message } as MessageEndEvent,
      ctx,
    )

    expect(fetchMock).toHaveBeenCalledOnce()
    const [url, opts] = fetchMock.mock.calls[0]!
    expect(url).toBe("http://127.0.0.1:9823/token-usage")
    expect(opts.method).toBe("POST")

    const body = JSON.parse(opts.body)
    expect(body).toMatchObject({
      sessionId: "session-abc",
      messageId: "msg-123",
      agent: "omp",
      model: "claude-sonnet-4-20250514",
      inputTokens: 100,
      outputTokens: 50,
      reasoningTokens: 0,
      cacheReadTokens: 20,
      cacheWriteTokens: 10,
    })
    expect(body.recordedAt).toBeTypeOf("number")
  })

  it("should ignore non-assistant messages", () => {
    extension(pi)

    const message = { role: "user", id: "msg-456" }
    const ctx = mockCtx()

    pi._emit(
      "message_end",
      { type: "message_end", message } as unknown as MessageEndEvent,
      ctx,
    )

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it("should ignore messages without usage data", () => {
    extension(pi)

    const message = { role: "assistant", id: "msg-789" }
    const ctx = mockCtx()

    pi._emit(
      "message_end",
      { type: "message_end", message } as unknown as MessageEndEvent,
      ctx,
    )

    expect(fetchMock).not.toHaveBeenCalled()
  })

  it("should use OMC_DAEMON_URL env var", () => {
    process.env.OMC_DAEMON_URL = "http://custom:9999"
    extension(pi)

    const message = mockAssistantMessage()
    const ctx = mockCtx()

    pi._emit(
      "message_end",
      { type: "message_end", message } as MessageEndEvent,
      ctx,
    )

    expect(fetchMock.mock.calls[0]![0]).toBe("http://custom:9999/token-usage")
  })

  it("should not throw when daemon is unreachable", () => {
    fetchMock.mockRejectedValue(new Error("ECONNREFUSED"))
    extension(pi)

    const message = mockAssistantMessage()
    const ctx = mockCtx()

    expect(() => {
      pi._emit(
        "message_end",
        { type: "message_end", message } as MessageEndEvent,
        ctx,
      )
    }).not.toThrow()
  })

  it("should log when daemon returns error status", async () => {
    fetchMock.mockResolvedValue({ ok: false, status: 500 })
    extension(pi)

    const message = mockAssistantMessage()
    const ctx = mockCtx()

    pi._emit(
      "message_end",
      { type: "message_end", message } as MessageEndEvent,
      ctx,
    )

    await vi.waitFor(() => expect(pi.logger.warn).toHaveBeenCalled())
  })

  it("should generate messageId when message has no id", () => {
    extension(pi)

    const message = mockAssistantMessage({ id: undefined })
    const ctx = mockCtx()

    pi._emit(
      "message_end",
      { type: "message_end", message } as unknown as MessageEndEvent,
      ctx,
    )

    const body = JSON.parse(fetchMock.mock.calls[0]![1].body)
    expect(body.messageId).toBeTypeOf("string")
    expect(body.messageId.length).toBeGreaterThan(0)
  })
})
