import { z } from "zod"

export const AuditConfigSchema = z.object({
  disabled: z.boolean().default(false),
  batch_size: z.number().int().min(1).max(100).default(20),
  push_interval_ms: z.number().int().min(5_000).max(300_000).default(30_000),
  retention_days: z.number().int().min(1).max(365).default(30),
})

export type AuditConfig = z.infer<typeof AuditConfigSchema>