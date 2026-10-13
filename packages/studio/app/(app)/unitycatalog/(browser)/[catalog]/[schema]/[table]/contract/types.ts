export type DataContractDifference = {
  severity?: string
  code?: string
  path?: string
  expected?: unknown
  actual?: unknown
}

export type OpenDataContractProperty = {
  id?: string
  name?: string
  logicalType?: string
  physicalType?: string
  required?: boolean
  description?: string
  partitionKeyPosition?: number
  [key: string]: unknown
}

export type OpenDataContractSchema = {
  id?: string
  name?: string
  logicalType?: string
  physicalType?: string
  physicalName?: string
  properties?: OpenDataContractProperty[]
  [key: string]: unknown
}

export type OpenDataContractDocument = {
  apiVersion?: string
  kind?: string
  id?: string
  name?: string
  version?: string
  status?: string
  domain?: string
  description?: string | Record<string, unknown>
  schema?: OpenDataContractSchema[]
  [key: string]: unknown
}

export type DataContract = {
  id: string
  catalog_name: string
  schema_name: string
  table_name: string
  version: string
  document: OpenDataContractDocument
  document_hash: string
  schema_hash: string
  created_by: string
  created_at: string
  sync_state: string
  unity_table_id?: string
  observed_schema_hash?: string
  differences: DataContractDifference[]
  checked_at?: string
}

export type ContractTarget = {
  catalog: string
  schema: string
  table: string
}

export type ContractActionResult =
  | { ok: true; data: DataContract | null }
  | { ok: false; error: string }
