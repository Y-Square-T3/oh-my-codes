import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import type {
  ExtensionAPI,
  ExtensionContext,
  MessageEndEvent,
} from "@oh-my-pi/pi-coding-agent/extensibility/extensions"
import type {
  DaemonModelsListResponse,
  DaemonCredentialsResponse,
} from "../src/types.js"
import extension from "../src/index.js"

function mockDaemonResponse(): {
  models: DaemonModelsListResponse
  credentials: DaemonCredentialsResponse
} {
  return {
    models: {
      providers: [
        {
          id: "omc-anthropic",
          name: "OMC Anthropic",
          env: [],
          modelCount: 2,
        },
      ],
      models: [
        {
          id: "claude-sonnet-4-20250514",
          providerId: "omc-anthropic",
          name: "Claude Sonnet 4",
          reasoning: true,
          attachment: true,
          modalitiesInput: ["text", "image"],
          modalitiesOutput: ["text"],
          costInput: 3,
          costOutput: 15,
          costCacheRead: 0.3,
          costCacheWrite: 3.75,
          limitContext: 200000,
          limitOutput: 8192,
        },
        {
          id: "claude-haiku-4-20250414",
          providerId: "omc-anthropic",
          name: "Claude Haiku 4",
          reasoning: false,
          modalitiesInput: ["text"],
          modalitiesOutput: ["text"],
          costInput: 0.25,
          costOutput: 1.25,
          limitContext: 200000,
          limitOutput: 8192,
        },
      ],
    },
    credentials: {
      apiKey: "omc-test-key-123",
      baseUrl: "https://api.omc.example.com",
      workspaceId: "ws-abc",
    },
  }
}

function mockPi() {
  type EventHandler = (event: unknown, ctx: unknown) => void
  const handlers = new Map<string, EventHandler[]>()
  const registeredProviders = new Map<string, unknown>()
  return {
    on: vi.fn((event: string, handler: EventHandler) => {
      if (!handlers.has(event)) handlers.set(event, [])
      handlers.get(event)!.push(handler)
    }),
    registerProvider: vi.fn((name: string, config: unknown) => {
      registeredProviders.set(name, config)
    }),
    unregisterProvider: vi.fn(),
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
    _registeredProviders: registeredProviders,
  } as unknown as ExtensionAPI & {
    _emit: (event: string, data: unknown, ctx: unknown) => void
    _handlers: Map<string, EventHandler[]>
    _registeredProviders: Map<string, unknown>
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
    responseId: "msg-123",
    ...overrides,
  }
}

function setupFetchMock(responses: Record<string, unknown>) {
  return vi.fn((url: string) => {
    for (const [pattern, data] of Object.entries(responses)) {
      if (url.includes(pattern)) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(data),
        })
      }
    }
    return Promise.resolve({ ok: false, status: 404 })
  })
}

describe("omc-pi extension", () => {
  let fetchMock: ReturnType<typeof vi.fn>
  let pi: ReturnType<typeof mockPi>

  beforeEach(() => {
    fetchMock = vi.fn()
    vi.stubGlobal("fetch", fetchMock)
    pi = mockPi()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    delete process.env.OMC_DAEMON_URL
  })

  describe("extension factory", () => {
    it("should export a default async function", () => {
      expect(typeof extension).toBe("function")
    })

    it("should return a promise", () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      const result = extension(pi)
      expect(result).toBeInstanceOf(Promise)
    })
  })

  describe("provider registration", () => {
    it("should fetch models and credentials from omcd", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(fetchMock).toHaveBeenCalledWith("http://127.0.0.1:9823/models")
      expect(fetchMock).toHaveBeenCalledWith(
        "http://127.0.0.1:9823/account/credentials",
      )
    })

    it("should call registerProvider for each provider from omcd", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(pi.registerProvider).toHaveBeenCalledTimes(1)
      expect(pi.registerProvider).toHaveBeenCalledWith(
        "omc-anthropic",
        expect.objectContaining({
          baseUrl: "https://api.omc.example.com",
          apiKey: "omc-test-key-123",
        }),
      )
    })

    it("should include x-workspace-id header when workspaceId is present", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      expect(config.headers).toEqual({ "x-workspace-id": "ws-abc" })
    })

    it("should not include headers when workspaceId is absent", async () => {
      const { models } = mockDaemonResponse()
      const credentials: DaemonCredentialsResponse = {
        apiKey: "omc-test-key-123",
        baseUrl: "https://api.omc.example.com",
      }
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      expect(config.headers).toBeUndefined()
    })

    it("should use fetchDynamicModels to return transformed models", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      expect(typeof config.fetchDynamicModels).toBe("function")

      const dynamicModels = await (
        config.fetchDynamicModels as (
          key: string | undefined,
        ) => Promise<unknown[]>
      )("test-key")
      expect(dynamicModels).toHaveLength(2)
      expect(dynamicModels[0]).toMatchObject({
        id: "claude-sonnet-4-20250514",
        name: "Claude Sonnet 4",
        reasoning: true,
        input: ["text", "image"],
        cost: { input: 3, output: 15, cacheRead: 0.3, cacheWrite: 3.75 },
        contextWindow: 200000,
        maxTokens: 8192,
      })
    })

    it("should register multiple providers when omcd returns multiple", async () => {
      const { models, credentials } = mockDaemonResponse()
      models.providers.push({
        id: "omc-openai",
        name: "OMC OpenAI",
        env: [],
        modelCount: 1,
      })
      models.models.push({
        id: "gpt-4o",
        providerId: "omc-openai",
        name: "GPT-4o",
        reasoning: false,
        modalitiesInput: ["text", "image"],
        modalitiesOutput: ["text"],
        costInput: 5,
        costOutput: 15,
        limitContext: 128000,
        limitOutput: 4096,
      })

      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(pi.registerProvider).toHaveBeenCalledTimes(2)
      expect(pi.registerProvider).toHaveBeenCalledWith(
        "omc-anthropic",
        expect.any(Object),
      )
      expect(pi.registerProvider).toHaveBeenCalledWith(
        "omc-openai",
        expect.any(Object),
      )
    })
  })

  describe("model transformation", () => {
    it("should map DaemonModelInfo to ProviderModelConfig correctly", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      const dynamicModels = await (
        config.fetchDynamicModels as (
          key: string | undefined,
        ) => Promise<Record<string, unknown>[]>
      )("test-key")

      const sonnet = dynamicModels.find(
        (m) => m.id === "claude-sonnet-4-20250514",
      )
      expect(sonnet).toMatchObject({
        id: "claude-sonnet-4-20250514",
        name: "Claude Sonnet 4",
        reasoning: true,
        input: ["text", "image"],
        cost: { input: 3, output: 15, cacheRead: 0.3, cacheWrite: 3.75 },
        contextWindow: 200000,
        maxTokens: 8192,
      })

      const haiku = dynamicModels.find(
        (m) => m.id === "claude-haiku-4-20250414",
      )
      expect(haiku).toMatchObject({
        id: "claude-haiku-4-20250414",
        name: "Claude Haiku 4",
        reasoning: false,
        input: ["text"],
        cost: { input: 0.25, output: 1.25, cacheRead: 0, cacheWrite: 0 },
        contextWindow: 200000,
        maxTokens: 8192,
      })
    })

    it("should default missing cost fields to 0", async () => {
      const { models, credentials } = mockDaemonResponse()
      models.models[0].costCacheRead = undefined
      models.models[0].costCacheWrite = undefined

      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      const dynamicModels = await (
        config.fetchDynamicModels as (
          key: string | undefined,
        ) => Promise<Record<string, unknown>[]>
      )("test-key")
      const sonnet = dynamicModels.find(
        (m) => m.id === "claude-sonnet-4-20250514",
      )
      expect(sonnet?.cost).toEqual({
        input: 3,
        output: 15,
        cacheRead: 0,
        cacheWrite: 0,
      })
    })

    it("should default missing limit fields to 0", async () => {
      const { models, credentials } = mockDaemonResponse()
      models.models[0].limitContext = undefined
      models.models[0].limitOutput = undefined

      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      const dynamicModels = await (
        config.fetchDynamicModels as (
          key: string | undefined,
        ) => Promise<Record<string, unknown>[]>
      )("test-key")
      const sonnet = dynamicModels.find(
        (m) => m.id === "claude-sonnet-4-20250514",
      )
      expect(sonnet?.contextWindow).toBe(0)
      expect(sonnet?.maxTokens).toBe(0)
    })

    it("should filter modalitiesInput to only text and image", async () => {
      const { models, credentials } = mockDaemonResponse()
      models.models[0].modalitiesInput = ["text", "image", "audio", "video"]

      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const config = pi._registeredProviders.get("omc-anthropic") as Record<
        string,
        unknown
      >
      const dynamicModels = await (
        config.fetchDynamicModels as (
          key: string | undefined,
        ) => Promise<Record<string, unknown>[]>
      )("test-key")
      const sonnet = dynamicModels.find(
        (m) => m.id === "claude-sonnet-4-20250514",
      )
      expect(sonnet?.input).toEqual(["text", "image"])
    })
  })

  describe("token tracking", () => {
    it("should register message_end handler", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(pi.on).toHaveBeenCalledWith("message_end", expect.any(Function))
    })

    it("should send token usage to daemon on message_end", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const message = mockAssistantMessage()
      const ctx = mockCtx("session-abc")

      pi._emit(
        "message_end",
        { type: "message_end", message } as unknown as MessageEndEvent,
        ctx,
      )

      const tokenUsageCalls = fetchMock.mock.calls.filter(
        (call) =>
          typeof call[0] === "string" && call[0].includes("/token-usage"),
      )
      expect(tokenUsageCalls).toHaveLength(1)
      const [url, opts] = tokenUsageCalls[0]
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
    })

    it("should ignore non-assistant messages", async () => {
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      const message = { role: "user", id: "msg-456" }
      const ctx = mockCtx()

      pi._emit(
        "message_end",
        { type: "message_end", message } as unknown as MessageEndEvent,
        ctx,
      )

      const tokenUsageCalls = fetchMock.mock.calls.filter(
        (call) =>
          typeof call[0] === "string" && call[0].includes("/token-usage"),
      )
      expect(tokenUsageCalls).toHaveLength(0)
    })
  })

  describe("error handling", () => {
    it("should log warning and skip provider injection when omcd is unreachable", async () => {
      fetchMock.mockRejectedValue(new Error("ECONNREFUSED"))

      await extension(pi)

      expect(pi.logger.warn).toHaveBeenCalled()
      expect(pi.registerProvider).not.toHaveBeenCalled()
    })

    it("should still register message_end handler when omcd is unreachable", async () => {
      fetchMock.mockRejectedValue(new Error("ECONNREFUSED"))

      await extension(pi)

      expect(pi.on).toHaveBeenCalledWith("message_end", expect.any(Function))
    })

    it("should log warning when omcd returns error status", async () => {
      fetchMock.mockResolvedValue({ ok: false, status: 500 })

      await extension(pi)

      expect(pi.logger.warn).toHaveBeenCalled()
      expect(pi.registerProvider).not.toHaveBeenCalled()
    })

    it("should log warning when no providers returned", async () => {
      const emptyModels: DaemonModelsListResponse = {
        providers: [],
        models: [],
      }
      const credentials: DaemonCredentialsResponse = {
        apiKey: "key",
        baseUrl: "https://api.example.com",
      }
      fetchMock = setupFetchMock({
        "/models": emptyModels,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(pi.logger.warn).toHaveBeenCalled()
      expect(pi.registerProvider).not.toHaveBeenCalled()
    })

    it("should use OMC_DAEMON_URL env var", async () => {
      process.env.OMC_DAEMON_URL = "http://custom:9999"
      const { models, credentials } = mockDaemonResponse()
      fetchMock = setupFetchMock({
        "/models": models,
        "/account/credentials": credentials,
      })
      vi.stubGlobal("fetch", fetchMock)

      await extension(pi)

      expect(fetchMock).toHaveBeenCalledWith("http://custom:9999/models")
      expect(fetchMock).toHaveBeenCalledWith(
        "http://custom:9999/account/credentials",
      )
    })
  })
})
