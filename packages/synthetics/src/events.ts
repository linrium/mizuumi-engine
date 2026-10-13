import type { RegionCode } from "./generators/index.js"
import type { SourceName } from "./generator.js"

export type ChangeOperation = "CREATE" | "UPDATE" | "DELETE"

export type IncrementalEvent = {
  id: string
  streamId: string
  region: RegionCode
  source: SourceName
  subject: string
  type: string
  time: string
  data: {
    operation: ChangeOperation
    entityType: "CUSTOMER"
    entityId: string
    entityVersion: number
    effectiveAt: string
    observedAt: string
    changedFields: string[]
    record: unknown
  }
}

export type IncrementalPublishSummary = {
  region: RegionCode
  topics: Record<SourceName, string>
  messages: Record<SourceName | "total", number>
}
