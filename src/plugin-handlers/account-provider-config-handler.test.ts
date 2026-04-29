/// <reference types="bun-types" />

import { Database } from "bun:sqlite"
import { describe, expect, mock, test } from "bun:test"
import { applyAccountProviderConfig } from "./account-provider-config-handler"

const mockDbInstances: Database[] = []

function createMockDatabase() {
  const db = {
    prepare: mock((query: string) => {
      if (query.includes("account_state")) {
        return {
          get: () => ({ active_account_id: "test-account-id" }),
        }
      }
      if (query.includes("SELECT id, name FROM providers")) {
        return {
          all: () => [
            { id: "anthropic", name: "Anthropic" },
            { id: "openai", name: "OpenAI" },
          ],
        }
      }
      if (query.includes("FROM models")) {
        return {
          all: () => [
            {
              id: "claude-sonnet-4-20250514",
              provider_id: "anthropic",
              limit_context: 200000,
              modalities_input: '["text", "image"]',
              modalities_output: '["text"]',
              attachment: 1,
              reasoning: 1,
              tool_call: 1,
              structured_output: 1,
              temperature: 1,
              open_weights: 0,
              interleaved_field: null,
              family: "claude",
              knowledge: null,
            },
            {
              id: "gpt-5",
              provider_id: "openai",
              limit_context: 100000,
              modalities_input: '["text"]',
              modalities_output: '["text"]',
              attachment: 0,
              reasoning: 1,
              tool_call: 1,
              structured_output: 0,
              temperature: 1,
              open_weights: 0,
              interleaved_field: null,
              family: null,
              knowledge: null,
            },
          ],
        }
      }
      return { all: () => [] }
    }),
    close: mock(() => {}),
  } as unknown as Database
  mockDbInstances.push(db)
  return db
}

describe("applyAccountProviderConfig", () => {
  test("does not modify config when no active account", () => {
    mock.module("bun:sqlite", () => ({
      Database: mock(() => {
        const db = {
          prepare: mock(() => ({
            get: () => ({ active_account_id: null }),
          })),
          close: mock(() => {}),
        } as unknown as Database
        mockDbInstances.push(db)
        return db
      }),
    }))

    const config: Record<string, unknown> = { provider: { existing: {} } }
    applyAccountProviderConfig({ config })

    expect(config.provider).toEqual({ existing: {} })
  })

  test("does not modify config when no providers for account", () => {
    mock.module("bun:sqlite", () => ({
      Database: mock(() => {
        const db = {
          prepare: mock((query: string) => {
            if (query.includes("account_state")) {
              return { get: () => ({ active_account_id: "test-account" }) }
            }
            if (query.includes("SELECT id, name FROM providers")) {
              return { all: () => [] }
            }
            return { all: () => [] }
          }),
          close: mock(() => {}),
        } as unknown as Database
        mockDbInstances.push(db)
        return db
      }),
    }))

    const config: Record<string, unknown> = { provider: { existing: {} } }
    applyAccountProviderConfig({ config })

    expect(config.provider).toEqual({ existing: {} })
  })

  test("injects provider config from database", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    expect(config.provider).toBeDefined()
    const provider = config.provider as Record<string, unknown>
    expect(provider["anthropic"]).toBeDefined()
    expect(provider["openai"]).toBeDefined()
  })

  test("maps context limits correctly", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>

    expect(claudeModel["limit"]).toEqual({ context: 200000 })
  })

  test("maps modalities correctly", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    const modalities = claudeModel["modalities"] as Record<string, unknown>

    expect(modalities["input"]).toEqual(["text", "image"])
    expect(modalities["output"]).toEqual(["text"])
  })

  test("maps all capabilities correctly", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

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

  test("detects image capability from modalities", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>
    const capabilities = claudeModel["capabilities"] as Record<string, unknown>
    const input = capabilities["input"] as Record<string, unknown>

    expect(input["image"]).toBe(true)
  })

  test("existing config overrides account config", () => {
    mock.module("bun:sqlite", () => ({
      Database: createMockDatabase,
    }))

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
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const anthropic = provider["anthropic"] as Record<string, unknown>
    const models = anthropic["models"] as Record<string, unknown>
    const claudeModel = models["claude-sonnet-4-20250514"] as Record<string, unknown>

    expect(claudeModel["limit"]).toEqual({ context: 999999 })
  })

  test("handles invalid JSON in modalities gracefully", () => {
    mock.module("bun:sqlite", () => ({
      Database: mock(() => {
        const db = {
          prepare: mock((query: string) => {
            if (query.includes("account_state")) {
              return { get: () => ({ active_account_id: "test-account" }) }
            }
            if (query.includes("SELECT id, name FROM providers")) {
              return { all: () => [{ id: "test", name: "Test" }] }
            }
            if (query.includes("FROM models")) {
              return {
                all: () => [
                  {
                    id: "model-1",
                    provider_id: "test",
                    limit_context: 100,
                    modalities_input: "invalid-json",
                    modalities_output: null,
                    attachment: 0,
                    reasoning: 0,
                    tool_call: 0,
                    structured_output: 0,
                    temperature: 0,
                    open_weights: 0,
                    interleaved_field: null,
                    family: null,
                    knowledge: null,
                  },
                ],
              }
            }
            return { all: () => [] }
          }),
          close: mock(() => {}),
        } as unknown as Database
        mockDbInstances.push(db)
        return db
      }),
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const testProvider = provider["test"] as Record<string, unknown>
    const models = testProvider["models"] as Record<string, unknown>
    const model1 = models["model-1"] as Record<string, unknown>

    expect(model1["limit"]).toEqual({ context: 100 })
    expect(model1["modalities"]).toBeUndefined()
  })

  test("maps interleaved field correctly", () => {
    mock.module("bun:sqlite", () => ({
      Database: mock(() => {
        const db = {
          prepare: mock((query: string) => {
            if (query.includes("account_state")) {
              return { get: () => ({ active_account_id: "test-account" }) }
            }
            if (query.includes("SELECT id, name FROM providers")) {
              return { all: () => [{ id: "test", name: "Test" }] }
            }
            if (query.includes("FROM models")) {
              return {
                all: () => [
                  {
                    id: "model-reasoning",
                    provider_id: "test",
                    limit_context: 1000,
                    modalities_input: null,
                    modalities_output: null,
                    attachment: 0,
                    reasoning: 1,
                    tool_call: 0,
                    structured_output: 0,
                    temperature: 0,
                    open_weights: 0,
                    interleaved_field: "reasoning_content",
                    family: null,
                    knowledge: null,
                  },
                ],
              }
            }
            return { all: () => [] }
          }),
          close: mock(() => {}),
        } as unknown as Database
        mockDbInstances.push(db)
        return db
      }),
    }))

    const config: Record<string, unknown> = {}
    applyAccountProviderConfig({ config })

    const provider = config.provider as Record<string, unknown>
    const testProvider = provider["test"] as Record<string, unknown>
    const models = testProvider["models"] as Record<string, unknown>
    const model1 = models["model-reasoning"] as Record<string, unknown>

    expect(model1["interleaved"]).toEqual({ field: "reasoning_content" })
  })
})