import { describe, it, expect } from "vitest"
import { ExamplePlugin } from "../src/index.js"

describe("ExamplePlugin", () => {
  it("should export a function", () => {
    expect(typeof ExamplePlugin).toBe("function")
  })

  it("should return hooks object", async () => {
    const mockClient = {
      app: {
        log: async () => {},
      },
    }

    const hooks = await ExamplePlugin({
      project: {} as any,
      client: mockClient as any,
      $: {} as any,
      directory: "/tmp",
      worktree: "/tmp",
    })

    expect(hooks).toHaveProperty("event")
    expect(typeof hooks.event).toBe("function")
  })
})
