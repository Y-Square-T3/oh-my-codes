/// <reference types="bun-types" />

import { Effect, Layer, Option } from "effect"
import { describe, expect, test } from "bun:test"
import { Service as Account, defaultLayer as accountDefaultLayer } from "../features/account"
import { ModelRepoService, modelRepoDefaultLayer } from "../features/model"
import { applyAccountProviderConfig } from "./account-provider-config-handler"

const testAccountId = "test-account-id" as any

const mockAccountServiceNoActive = Layer.succeed(Account, {
  active: () => Effect.succeed(Option.none()),
  activeWithToken: () => Effect.succeed(Option.none()),
})

const mockAccountServiceWithAccount = Layer.succeed(Account, {
  active: () =>
    Effect.succeed(
      Option.some({
        id: testAccountId,
        email: "test@example.com",
        url: "https://example.com",
        activeWorkspaceId: null,
      }),
    ),
  activeWithToken: () =>
    Effect.succeed(
      Option.some({
        account: {
          id: testAccountId,
          email: "test@example.com",
          url: "https://example.com",
          activeWorkspaceId: "test-workspace-id",
        },
        accessToken: "test-access-token",
      }),
    ),
})

const mockAccountServiceWithAccountNoWorkspace = Layer.succeed(Account, {
  active: () =>
    Effect.succeed(
      Option.some({
        id: testAccountId,
        email: "test@example.com",
        url: "https://example.com",
        activeWorkspaceId: null,
      }),
    ),
  activeWithToken: () =>
    Effect.succeed(
      Option.some({
        account: {
          id: testAccountId,
          email: "test@example.com",
          url: "https://example.com",
          activeWorkspaceId: null,
        },
        accessToken: "test-access-token-no-workspace",
      }),
    ),
})

const mockModelRepoEmpty = Layer.succeed(ModelRepoService, {
  listProviders: () => Effect.succeed([]),
  listModels: () => Effect.succeed([]),
})

const testProviders = [
  { id: "anthropic", name: "Anthropic" },
  { id: "openai", name: "OpenAI" },
] as const

const testModels = [
  {
    id: "claude-sonnet-4-20250514",
    providerId: "anthropic",
    limitContext: 200000,
    modalitiesInput: '["text", "image"]',
    modalitiesOutput: '["text"]',
    attachment: 1,
    reasoning: 1,
    toolCall: 1,
    structuredOutput: 1,
    temperature: 1,
    openWeights: 0,
    interleavedField: null,
    family: "claude",
    knowledge: null,
  },
  {
    id: "gpt-5",
    providerId: "openai",
    limitContext: 100000,
    modalitiesInput: '["text"]',
    modalitiesOutput: '["text"]',
    attachment: 0,
    reasoning: 1,
    toolCall: 1,
    structuredOutput: 0,
    temperature: 1,
    openWeights: 0,
    interleavedField: null,
    family: null,
    knowledge: null,
  },
] as const

const mockModelRepoWithData = Layer.succeed(ModelRepoService, {
  listProviders: () => Effect.succeed([...testProviders]),
  listModels: () => Effect.succeed([...testModels]),
})

function makeTestLayer(
  accountMock: typeof mockAccountServiceNoActive,
  repoMock: typeof mockModelRepoEmpty,
) {
  return Layer.mergeAll(
    accountDefaultLayer,
    modelRepoDefaultLayer,
    accountMock,
    repoMock,
  )
}

describe("applyAccountProviderConfig", () => {
  test("does not modify config when no active account", async () => {
    const layer = makeTestLayer(mockAccountServiceNoActive, mockModelRepoEmpty)

    const config: Record<string, unknown> = { provider: { existing: {} } }
    await applyAccountProviderConfig({ config, layer })

    expect(config.provider).toEqual({ existing: {} })
  })

  test("does not modify config when no providers for account", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoEmpty)

    const config: Record<string, unknown> = { provider: { existing: {} } }
    await applyAccountProviderConfig({ config, layer })

    expect(config.provider).toEqual({ existing: {} })
  })

  test("injects provider config from database", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    expect(config.provider).toBeDefined()
    const provider = config.provider as Record<string, unknown>
    expect(provider["anthropic"]).toBeDefined()
    expect(provider["openai"]).toBeDefined()
  })

  test("maps context limits correctly", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>

    expect(claudeModel["limit"]).toEqual({ context: 200000 })
  })

  test("maps modalities correctly", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    const modalities = claudeModel["modalities"] as Record<string, unknown>

    expect(modalities["input"]).toEqual(["text", "image"])
    expect(modalities["output"]).toEqual(["text"])
  })

  test("maps all capabilities correctly", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    const capabilities = claudeModel["capabilities"] as Record<string, unknown>

    expect(capabilities["attachment"]).toBe(true)
    expect(capabilities["reasoning"]).toBe(true)
    expect(capabilities["tool_call"]).toBe(true)
    expect(capabilities["structured_output"]).toBe(true)
    expect(capabilities["temperature"]).toBe(true)
    expect(capabilities["open_weights"]).toBeUndefined()
  })

  test("detects image capability from modalities", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    const capabilities = claudeModel["capabilities"] as Record<string, unknown>
    const input = capabilities["input"] as Record<string, unknown>

    expect(input["image"]).toBe(true)
  })

  test("existing config overrides account config", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {
      provider: {
        anthropic: {
          models: {
            "claude-sonnet-4-20250514": {
              limit: { context: 999999 },
            },
          },
        },
      },
    }
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>

    expect(claudeModel["limit"]).toEqual({ context: 999999 })
  })

  test("handles invalid JSON in modalities gracefully", async () => {
    const invalidJsonRepo = Layer.succeed(ModelRepoService, {
      listProviders: () =>
        Effect.succeed([{ id: "test", name: "Test" } as const]),
      listModels: () =>
        Effect.succeed([
          {
            id: "model-1",
            providerId: "test",
            limitContext: 100,
            modalitiesInput: "invalid-json",
            modalitiesOutput: null,
            attachment: 0,
            reasoning: 0,
            toolCall: 0,
            structuredOutput: 0,
            temperature: 0,
            openWeights: 0,
            interleavedField: null,
            family: null,
            knowledge: null,
          },
        ] as const),
    })
    const layer = makeTestLayer(mockAccountServiceWithAccount, invalidJsonRepo)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const testProvider = provider["test"] as Record<string, unknown>
    const models = testProvider["models"] as Record<string, unknown>
    const model1 = models["model-1"] as Record<string, unknown>

    expect(model1["limit"]).toEqual({ context: 100 })
    expect(model1["modalities"]).toBeUndefined()
  })

  test("maps interleaved field correctly", async () => {
    const interleavedRepo = Layer.succeed(ModelRepoService, {
      listProviders: () =>
        Effect.succeed([{ id: "test", name: "Test" } as const]),
      listModels: () =>
        Effect.succeed([
          {
            id: "model-reasoning",
            providerId: "test",
            limitContext: 1000,
            modalitiesInput: null,
            modalitiesOutput: null,
            attachment: 0,
            reasoning: 1,
            toolCall: 0,
            structuredOutput: 0,
            temperature: 0,
            openWeights: 0,
            interleavedField: "reasoning_content",
            family: null,
            knowledge: null,
          },
        ] as const),
    })
    const layer = makeTestLayer(mockAccountServiceWithAccount, interleavedRepo)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const testProvider = provider["test"] as Record<string, unknown>
    const models = testProvider["models"] as Record<string, unknown>
    const model1 = models["model-reasoning"] as Record<string, unknown>

    expect(model1["interleaved"]).toEqual({ field: "reasoning_content" })
  })

  test("injects apiKey, baseURL, and x-workspace-id header into provider config options", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>

    expect(anthropic["apiKey"]).toBeUndefined()
    expect((anthropic["options"] as Record<string, unknown>)["apiKey"]).toBe("test-access-token")
    expect((anthropic["options"] as Record<string, unknown>)["baseURL"]).toBe("https://example.com/api/v2")
    expect((anthropic["options"] as Record<string, unknown>)["headers"]).toEqual({
      "x-workspace-id": "test-workspace-id",
    })
  })

  test("does not add x-workspace-id header when activeWorkspaceId is null", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccountNoWorkspace, mockModelRepoWithData)

    const config: Record<string, unknown> = {}
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>

    expect(anthropic["apiKey"]).toBeUndefined()
    expect((anthropic["options"] as Record<string, unknown>)["apiKey"]).toBe("test-access-token-no-workspace")
    expect((anthropic["options"] as Record<string, unknown>)["baseURL"]).toBe("https://example.com/api/v2")
    expect((anthropic["options"] as Record<string, unknown>)["headers"]).toBeUndefined()
  })

  test("user config overrides are preserved via spread order", async () => {
    const layer = makeTestLayer(mockAccountServiceWithAccount, mockModelRepoWithData)

    const config: Record<string, unknown> = {
      provider: {
        anthropic: {
          options: {
            apiKey: "user-api-key",
            baseURL: "https://user.example.com",
            headers: {
              "x-workspace-id": "user-workspace",
            },
          },
          models: {
            "claude-sonnet-4-20250514": {
              limit: { context: 999999 },
            },
          },
        },
      },
    }
    await applyAccountProviderConfig({ config, layer })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>

    expect(anthropic["apiKey"]).toBeUndefined()
    expect((anthropic["options"] as Record<string, unknown>)["apiKey"]).toBe("user-api-key")
    expect((anthropic["options"] as Record<string, unknown>)["baseURL"]).toBe("https://user.example.com")
    expect((anthropic["options"] as Record<string, unknown>)["headers"]).toEqual({
      "x-workspace-id": "user-workspace",
    })
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    expect(claudeModel["limit"]).toEqual({ context: 999999 })
  })
})