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
  primaryKey?: boolean
  quality?: OpenDataQualityRule[]
  customProperties?: OpenDataContractCustomProperty[]
  [key: string]: unknown
}

export type OpenDataContractSchema = {
  id?: string
  name?: string
  logicalType?: string
  physicalType?: string
  physicalName?: string
  properties?: OpenDataContractProperty[]
  quality?: OpenDataQualityRule[]
  customProperties?: OpenDataContractCustomProperty[]
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
  dataProduct?: string
  description?: string | Record<string, unknown>
  team?: OpenDataContractTeam
  slaProperties?: OpenDataContractSla[]
  customProperties?: OpenDataContractCustomProperty[]
  schema?: OpenDataContractSchema[]
  [key: string]: unknown
}

export type OpenDataQualityRule = {
  id?: string
  name?: string
  type?: string
  metric?: string
  description?: string
  severity?: string
  dimension?: string
  unit?: string
  [key: string]: unknown
}

export type OpenDataContractCustomProperty = {
  id?: string
  property?: string
  value?: unknown
  description?: string
}

export type OpenDataContractTeamMember = {
  id?: string
  username: string
  name?: string
  role?: string
  description?: string
}

export type OpenDataContractTeam = {
  id?: string
  name?: string
  description?: string
  members?: OpenDataContractTeamMember[]
}

export type OpenDataContractSla = {
  id?: string
  property: string
  value: unknown
  valueExt?: unknown
  unit?: string
  element?: string
  driver?: string
  description?: string
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

export type DataQualityResult = {
  rule_id: string
  rule_scope: string
  rule_type: string
  metric: string
  severity: string
  status: string
  metric_value?: unknown
  expectation: Record<string, unknown>
  duration_ms: number
  message?: string
}

export type DataQualityRun = {
  id: string
  contract_id: string
  contract_version: string
  engine: string
  status: string
  trigger_type: string
  triggered_by: string
  started_at: string
  completed_at?: string
  passed_count: number
  failed_count: number
  error_count: number
  error?: string
  results: DataQualityResult[]
}

export type QualityRunsActionResult =
  | { ok: true; data: DataQualityRun[] }
  | { ok: false; error: string }

export type QualityRunActionResult =
  | { ok: true; data: DataQualityRun }
  | { ok: false; error: string }
