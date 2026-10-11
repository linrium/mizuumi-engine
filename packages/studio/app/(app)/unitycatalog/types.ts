export type Column = {
  name: string
  type_text?: string | null
  type_name?: string | null
  position?: number | null
  nullable?: boolean | null
  comment?: string | null
  partition_index?: number | null
}

export type Table = {
  name: string
  catalog_name: string
  schema_name: string
  table_type?: string | null
  data_source_format?: string | null
  table_id?: string | null
  owner?: string | null
  created_at?: number | null
  updated_at?: number | null
  storage_location?: string | null
  columns?: Column[]
  properties?: Record<string, string>
}

export type Schema = {
  name: string
  full_name?: string | null
  catalog_name: string
  comment?: string | null
  owner?: string | null
  created_at?: number | null
  schema_id?: string | null
  storage_root?: string | null
  tables?: Table[]
}

export type Catalog = {
  name: string
  comment?: string | null
  owner?: string | null
  created_at?: number | null
  updated_at?: number | null
  id?: string | null
  storage_root?: string | null
  schemas?: Schema[]
}

export type UnityCatalogData = { catalogs?: Catalog[] }

export function formatDate(ms?: number | null) {
  if (!ms) return "—"
  return new Date(ms).toLocaleString()
}
