import { defineConfig } from "drizzle-kit"

export default defineConfig({
  dialect: "sqlite",
  schema: "./src/features/database/schema.ts",
  out: "./src/features/database/migrations",
  dbCredentials: {
    url: "file:oh-my-codes.db",
  },
})
