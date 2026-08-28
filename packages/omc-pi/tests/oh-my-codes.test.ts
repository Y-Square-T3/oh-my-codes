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

  it("should return hooks with event and config handlers", async () => {
    const hooks = await OhMyCodesPlugin(mockInput())
    expect(hooks).toHaveProperty("event")
    expect(typeof hooks.event).toBe("function")
    expect(hooks).toHaveProperty("config")
    expect(typeof hooks.config).toBe("function")
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
        sessionId: "sess-1",
        messageId: "msg-1",
        agent: "coder",
        model: "claude-sonnet-4-20250514",
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
        sessionId: "sess-2",
        messageId: "msg-2",
        model: "gpt-4o",
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

  describe("config hook", () => {
    const mockModelsResponse = {
      providers: [
        {
          id: "anthropic",
          name: "Anthropic",
          api: "https://api.anthropic.com",
          npm: null,
          env: ["ANTHROPIC_API_KEY"],
          modelCount: 1,
        },
      ],
      models: [
        {
          id: "claude-sonnet-4-20250514",
          providerId: "anthropic",
          name: "Claude Sonnet 4",
          family: "claude",
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
      accountEmail: "test@example.com",
      accountUrl: "https://api.omc.ai",
    }

    const mockCredentialsResponse = {
      apiKey: "test-oauth-token",
      baseUrl: "https://api.omc.ai/api/v2",
      workspaceId: "ws-123",
    }

    it("should have config hook", async () => {
      const hooks = await OhMyCodesPlugin(mockInput())
      expect(hooks).toHaveProperty("config")
      expect(typeof hooks.config).toBe("function")
    })

    it("should inject providers into config", async () => {
      fetchMock.mockImplementation((url: string) => {
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

      const hooks = await OhMyCodesPlugin(mockInput())
      const config: Record<string, unknown> = {}
      await hooks.config!(config as never)

      expect(config.provider).toBeDefined()
      const providers = config.provider as Record<string, unknown>
      expect(providers.anthropic).toBeDefined()

      const anthropic = providers.anthropic as Record<string, unknown>
      expect(anthropic.models).toBeDefined()
      expect(anthropic.options).toBeDefined()

      const models = anthropic.models as Record<string, unknown>
      expect(models["claude-sonnet-4-20250514"]).toBeDefined()

      const options = anthropic.options as Record<string, unknown>
      expect(options.apiKey).toBe("test-oauth-token")
      expect(options.baseURL).toBe("https://api.omc.ai/api/v2")
      expect(options.headers).toEqual({ "x-workspace-id": "ws-123" })
    })

    it("should not override user's existing providers", async () => {
      fetchMock.mockImplementation((url: string) => {
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

      const hooks = await OhMyCodesPlugin(mockInput())
      const config: Record<string, unknown> = {
        provider: {
          anthropic: {
            models: {},
            options: { apiKey: "user-key" },
          },
        },
      }
      await hooks.config!(config as never)

      const providers = config.provider as Record<string, unknown>
      const anthropic = providers.anthropic as Record<string, unknown>
      const options = anthropic.options as Record<string, unknown>
      expect(options.apiKey).toBe("user-key")
    })

    it("should handle missing models gracefully", async () => {
      fetchMock.mockImplementation((url: string) => {
        if (url.includes("/models")) {
          return Promise.resolve({ ok: false, status: 500 })
        }
        if (url.includes("/account/credentials")) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockCredentialsResponse),
          })
        }
        return Promise.resolve({ ok: true })
      })

      const input = mockInput()
      const hooks = await OhMyCodesPlugin(input)
      const config: Record<string, unknown> = {}
      await hooks.config!(config as never)

      expect(config.provider).toBeUndefined()
      expect(input.client.app.log).toHaveBeenCalled()
    })

    it("should handle missing credentials gracefully", async () => {
      fetchMock.mockImplementation((url: string) => {
        if (url.includes("/models")) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(mockModelsResponse),
          })
        }
        if (url.includes("/account/credentials")) {
          return Promise.resolve({ ok: false, status: 401 })
        }
        return Promise.resolve({ ok: true })
      })

      const input = mockInput()
      const hooks = await OhMyCodesPlugin(input)
      const config: Record<string, unknown> = {}
      await hooks.config!(config as never)

      expect(config.provider).toBeUndefined()
      expect(input.client.app.log).toHaveBeenCalled()
    })

    it("should skip injection when no providers available", async () => {
      fetchMock.mockImplementation((url: string) => {
        if (url.includes("/models")) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                providers: [],
                models: [],
                accountEmail: null,
                accountUrl: null,
              }),
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

      const input = mockInput()
      const hooks = await OhMyCodesPlugin(input)
      const config: Record<string, unknown> = {}
      await hooks.config!(config as never)

      expect(config.provider).toBeUndefined()
      expect(input.client.app.log).toHaveBeenCalled()
    })

    it("should build correct model config with capabilities", async () => {
      fetchMock.mockImplementation((url: string) => {
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

      const hooks = await OhMyCodesPlugin(mockInput())
      const config: Record<string, unknown> = {}
      await hooks.config!(config as never)

      const providers = config.provider as Record<string, unknown>
      const anthropic = providers.anthropic as Record<string, unknown>
      const models = anthropic.models as Record<string, unknown>
      const model = models["claude-sonnet-4-20250514"] as Record<
        string,
        unknown
      >

      expect(model.limit).toEqual({ context: 200000, output: 16384 })
      expect(model.modalities).toEqual({
        input: ["text", "image"],
        output: ["text"],
      })
      expect(model.capabilities).toEqual({
        attachment: true,
        reasoning: true,
        tool_call: true,
        temperature: true,
        input: { image: true },
      })
    })
  })
})
