import { describe, expect, test } from "bun:test"

import { validateNonTuiArgs } from "./install-validators"

describe("validateNonTuiArgs", () => {
  test("always returns valid for simplified install args", () => {
    // #when
    const result = validateNonTuiArgs()

    // #then
    expect(result.valid).toBe(true)
    expect(result.errors).toHaveLength(0)
  })
})
