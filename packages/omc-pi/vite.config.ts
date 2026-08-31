import { defineConfig } from "vite"
import dts from "vite-plugin-dts"
import { resolve } from "path"

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, "src/index.ts"),
      formats: ["es"],
      fileName: "index",
    },
    rollupOptions: {
      external: ["@oh-my-pi/pi-coding-agent/extensibility/extensions"],
    },
  },
  plugins: [dts({ rollupTypes: true })],
})
