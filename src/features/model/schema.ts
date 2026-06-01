export { modelRecords, providers } from "../database/schema"

import { modelRecords, providers } from "../database/schema"

export type ProviderRow = typeof providers.$inferSelect
export type ModelRow = typeof modelRecords.$inferSelect
