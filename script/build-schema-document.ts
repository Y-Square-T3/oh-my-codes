import { z } from "zod"
import { OhMyCodesConfigSchema } from "../src/config/schema"

export function createOhMyCodesJsonSchema(): Record<string, unknown> {
  const jsonSchema = z.toJSONSchema(OhMyCodesConfigSchema, {
    target: "draft-7",
    unrepresentable: "any",
  }) as Record<string, unknown>

  return {
    $schema: "http://json-schema.org/draft-07/schema#",
    $id: "https://raw.githubusercontent.com/vibration-autos/oh-my-codes/dev/assets/oh-my-codes.schema.json",
    title: "Oh My OpenCode Configuration",
    description: "Configuration schema for oh-my-codes plugin",
    ...jsonSchema,
  }
}
