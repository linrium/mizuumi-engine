"use client"

import { IconTable } from "@tabler/icons-react"
import { use } from "react"
import { Badge } from "@/components/ui/badge"
import { useBrowser } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import { HeaderBlock, NotFound } from "@/app/(app)/unitycatalog/(browser)/detail"
import { TableNav } from "@/app/(app)/unitycatalog/(browser)/[catalog]/[schema]/[table]/table-nav"

export default function TableLayout({
  params,
  children,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
  children: React.ReactNode
}) {
  const {
    catalog: catalogParam,
    schema: schemaParam,
    table: tableParam,
  } = use(params)
  const { catalogs, isPending } = useBrowser()
  const catalog = catalogs.find((c) => c.name === catalogParam)
  const schema = catalog?.schemas?.find((s) => s.name === schemaParam)
  const table = schema?.tables?.find((t) => t.name === tableParam)

  if (!catalog || !schema || !table) {
    return (
      <NotFound
        message={
          isPending
            ? "Loading…"
            : `Table "${catalogParam}.${schemaParam}.${tableParam}" not found.`
        }
      />
    )
  }

  return (
    <div className="space-y-3">
      <HeaderBlock
        icon={<IconTable className="size-4" />}
        kind="Table"
        title={`${catalog.name}.${schema.name}.${table.name}`}
        badges={
          <>
            {table.table_type ? (
              <Badge variant="secondary">{table.table_type}</Badge>
            ) : null}
            {table.data_source_format ? (
              <Badge variant="outline">{table.data_source_format}</Badge>
            ) : null}
          </>
        }
      />
      <TableNav
        catalog={catalog.name}
        schema={schema.name}
        table={table.name}
      />
      {children}
    </div>
  )
}
