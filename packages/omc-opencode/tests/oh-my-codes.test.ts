import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { OhMyCodesPlugin } from "../src/index.js"

function mockClient() {
  return {
    app: {
      log: vi.fn().mockResolvedValue(undefined),
    },
  }
}

function mockInput() {
  return {
    project: {} as never,
    client: mockClient() as never,
    $: {} as never,
    directory: "/tmp",
    worktree: "/tmp",
  }
}

describe("OhMyCodesPlugin", () => {
  let fetchMock: ReturnType<typeof vi.fn>

  beforeEach(() => {
    fetchMock = vi.fn().mockResolvedValue({ ok: true })
    vi.stubGlobal("fetch", fetchMock)
  })

  afterEach(() => {
    vi.restoreAllMocks()
    delete process.env.OMC_DAEMON_URL
  })

  it("should export a function", () => {
    expect(typeof OhMyCodesPlugin).toBe("function")
  })

  it("should return hooks with event handler", async () => {
    const hooks = await OhMyCodesPlugin(mockInput())
    expect(hooks).toHaveProperty("event")
    expect(typeof hooks.event).toBe("function")
  })

  describe("V1 message.updated", () => {
    it("should send record for finished assistant message", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "message.updated",
          properties: {
            info: {
              role: "assistant",
              finish: true,
              sessionID: "sess-1",
              id: "msg-1",
              agent: "coder",
              providerID: "anthropic",
              modelID: "claude-sonnet-4-20250514",
              tokens: {
                input: 100,
                output: 200,
                reasoning: 10,
                cache: { read: 50, write: 30 },
              },
            },
          },
        },
      } as never)

      expect(fetchMock).toHaveBeenCalledOnce()
      const [url, opts] = fetchMock.mock.calls[0]!
      expect(url).toContain("/token-usage")
      const body = JSON.parse(opts.body)
      expect(body).toMatchObject({
        client: "opencode",
        sessionId: "sess-1",
        messageId: "msg-1",
        agent: "coder",
        providerId: "anthropic",
        modelId: "claude-sonnet-4-20250514",
        inputTokens: 100,
        outputTokens: 200,
        reasoningTokens: 10,
        cacheReadTokens: 50,
        cacheWriteTokens: 30,
      })
    })

    it("should ignore user messages", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "message.updated",
          properties: {
            info: { role: "user", finish: true, sessionID: "s", tokens: {} },
          },
        },
      } as never)
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it("should ignore unfinished assistant messages", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "message.updated",
          properties: {
            info: {
              role: "assistant",
              finish: false,
              sessionID: "s",
              tokens: { input: 1 },
            },
          },
        },
      } as never)
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it("should ignore compaction agent messages", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "message.updated",
          properties: {
            info: {
              role: "assistant",
              finish: true,
              sessionID: "s",
              id: "m",
              agent: "compaction",
              tokens: { input: 1, output: 2 },
            },
          },
        },
      } as never)
      expect(fetchMock).not.toHaveBeenCalled()
    })
  })

  describe("V2 session.next.step.ended", () => {
    it("should send record for step ended event", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "session.next.step.ended",
          properties: {
            sessionID: "sess-2",
            assistantMessageID: "msg-2",
            agent: "coder",
            providerID: "openai",
            modelID: "gpt-4o",
            tokens: {
              input: 500,
              output: 300,
              reasoning: 0,
              cache: { read: 200, write: 100 },
            },
          },
        },
      } as never)

      expect(fetchMock).toHaveBeenCalledOnce()
      const body = JSON.parse(fetchMock.mock.calls[0]![1].body)
      expect(body).toMatchObject({
        client: "opencode",
        sessionId: "sess-2",
        messageId: "msg-2",
        providerId: "openai",
        modelId: "gpt-4o",
        inputTokens: 500,
        outputTokens: 300,
      })
    })

    it("should ignore compaction agent steps", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "session.next.step.ended",
          properties: {
            sessionID: "s",
            assistantMessageID: "m",
            agent: "summarize",
            tokens: { input: 1, output: 2 },
          },
        },
      } as never)
      expect(fetchMock).not.toHaveBeenCalled()
    })
  })

  describe("daemon URL", () => {
    it("should use OMC_DAEMON_URL env var", async () => {
      process.env.OMC_DAEMON_URL = "http://custom:9999"
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "session.next.step.ended",
          properties: {
            sessionID: "s",
            assistantMessageID: "m",
            tokens: { input: 1, output: 2 },
          },
        },
      } as never)

      expect(fetchMock.mock.calls[0]![0]).toBe("http://custom:9999/token-usage")
    })

    it("should default to localhost:9823", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({
        event: {
          type: "session.next.step.ended",
          properties: {
            sessionID: "s",
            assistantMessageID: "m",
            tokens: { input: 1, output: 2 },
          },
        },
      } as never)

      expect(fetchMock.mock.calls[0]![0]).toBe(
        "http://127.0.0.1:9823/token-usage",
      )
    })
  })

  describe("error handling", () => {
    it("should not throw when daemon is unreachable", async () => {
      fetchMock.mockRejectedValue(new Error("ECONNREFUSED"))
      const hooks = await OhMyCodesPlugin(mockInput())

      await expect(
        hooks.event!({
          event: {
            type: "session.next.step.ended",
            properties: {
              sessionID: "s",
              assistantMessageID: "m",
              tokens: { input: 1, output: 2 },
            },
          },
        } as never),
      ).resolves.toBeUndefined()
    })

    it("should log when daemon returns error status", async () => {
      fetchMock.mockResolvedValue({ ok: false, status: 500 })
      const input = mockInput()
      const hooks = await OhMyCodesPlugin(input)

      await hooks.event!({
        event: {
          type: "session.next.step.ended",
          properties: {
            sessionID: "s",
            assistantMessageID: "m",
            tokens: { input: 1, output: 2 },
          },
        },
      } as never)

      expect(input.client.app.log).toHaveBeenCalled()
    })
  })

  describe("unrelated events", () => {
    it("should ignore non-token-usage events", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      await hooks.event!({ event: { type: "session.created" } } as never)
      await hooks.event!({ event: { type: "file.edited" } } as never)
      await hooks.event!({ event: { type: "plugin.added" } } as never)
      expect(fetchMock).not.toHaveBeenCalled()
    })
  })
})
