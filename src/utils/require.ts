import { createRequire } from "node:module"

// eslint-disable-next-line @typescript-eslint/no-explicit-any
declare const __filename: string | undefined

/**
 * Pre-configured require function that works in both ESM and CJS contexts.
 * - ESM (dev mode, esbuild ESM bundle): uses import.meta.url
 * - CJS (pkg binary): uses __filename
 * - Fallback: uses process.cwd()
 */
export const require = createRequire(
  typeof import.meta?.url !== "undefined"
    ? import.meta.url
    : typeof __filename !== "undefined"
      ? __filename
      : process.cwd() + "/"
)
