import { afterEach, beforeEach, describe, expect, it, mock } from "bun:test"
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { getCachedVersion } from "./cached-version"

// Hold mutable mock state so beforeEach can swap the cache root for each test.
const mockState: { candidates: string[] } = { candidates: [] }

mock.module("../constants", () => ({
  INSTALLED_PACKAGE_JSON_CANDIDATES: new Proxy([], {
    get(_, prop) {
      const current = mockState.candidates
      // Forward array methods/properties to the mutable candidates list
      // so getCachedVersion's `for (... of ...)` sees fresh data per test.
      const value = (current as unknown as Record<PropertyKey, unknown>)[prop]
      if (typeof value === "function") {
        return (value as (...args: unknown[]) => unknown).bind(current)
      }
      return value
    },
  }),
}))

mock.module("./package-json-locator", () => ({
  findPackageJsonUp: () => null,
}))

describe("getCachedVersion (GH-3257)", () => {
  let cacheRoot: string

  beforeEach(() => {
    cacheRoot = mkdtempSync(join(tmpdir(), "omo-cached-version-"))
    mockState.candidates = [
      join(cacheRoot, "node_modules", "oh-my-codes", "package.json"),
      join(cacheRoot, "node_modules", "oh-my-codes", "package.json"),
    ]
  })

  afterEach(() => {
    rmSync(cacheRoot, { recursive: true, force: true })
    mockState.candidates = []
  })

  it("returns the version when the package is installed under oh-my-codes", () => {
    const pkgDir = join(cacheRoot, "node_modules", "oh-my-codes")
    mkdirSync(pkgDir, { recursive: true })
    writeFileSync(
      join(pkgDir, "package.json"),
      JSON.stringify({ name: "oh-my-codes", version: "3.16.0" }),
    )

    expect(getCachedVersion()).toBe("3.16.0")
  })

  it("returns the version when the package is installed under oh-my-codes", () => {
    // GH-3257: npm users who install the aliased `oh-my-codes` package get
    // node_modules/oh-my-codes/package.json, not the canonical oh-my-codes
    // path. The cached version resolver must check both.
    const pkgDir = join(cacheRoot, "node_modules", "oh-my-codes")
    mkdirSync(pkgDir, { recursive: true })
    writeFileSync(
      join(pkgDir, "package.json"),
      JSON.stringify({ name: "oh-my-codes", version: "3.16.0" }),
    )

    expect(getCachedVersion()).toBe("3.16.0")
  })

  it("returns null when neither candidate exists and fallbacks find nothing", () => {
    expect(getCachedVersion()).toBeNull()
  })
})
