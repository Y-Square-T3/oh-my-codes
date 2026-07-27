import type { Plugin } from "@opencode-ai/plugin"

export const ExamplePlugin: Plugin = async ({ client }) => {
  return {
    event: async ({ event }) => {
      await client.app.log({
        body: {
          service: "oh-my-codes-opencode",
          level: "info",
          message: `Event: ${event.type}`,
        },
      })
    },
  }
}
