"use client"

import { IconTable } from "@tabler/icons-react"
import Link from "next/link"
import { use } from "react"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useBrowser } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import { MetaGrid, NotFound } from "@/app/(app)/unitycatalog/(browser)/detail"
import { formatDate } from "@/app/(app)/unitycatalog/types"

export default function SchemaPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string }>
}) {
  const { catalog: catalogParam, schema: schemaParam } = use(params)
  const { catalogs, isPending } = useBrowser()
  const catalog = catalogs.find((c) => c.name === catalogParam)
  const schema = catalog?.schemas?.find((s) => s.name === schemaParam)

  if (!catalog || !schema) {
    return (
      <NotFound
        message={
          isPending
            ? "Loading…"
            : `Schema "${catalogParam}.${schemaParam}" not found.`
        }
      />
    )
  }

  return (
    <div className="space-y-6">
      <MetaGrid
        items={[
          ["Owner", schema.owner ?? "—"],
          ["Schema ID", schema.schema_id ?? "—"],
          ["Storage root", schema.storage_root ?? "—"],
          ["Created", formatDate(schema.created_at)],
          ["Tables", String(schema.tables?.length ?? 0)],
        ]}
      />
      <section>
        <h3 className="mb-2 text-xs font-semibold">Tables</h3>
        <div className="grid gap-2 sm:grid-cols-2">
          {(schema.tables ?? []).map((table) => (
            <Link
              key={table.name}
              href={`/unitycatalog/${encodeURIComponent(catalog.name)}/${encodeURIComponent(schema.name)}/${encodeURIComponent(table.name)}`}
            >
              <Card size="sm" className="hover:bg-muted/40">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-xs">
                    <IconTable className="size-3.5 text-muted-foreground" />
                    {table.name}
                  </CardTitle>
                  <CardDescription className="flex flex-wrap gap-1.5">
                    {table.table_type ? (
                      <Badge variant="secondary">{table.table_type}</Badge>
                    ) : null}
                    {table.data_source_format ? (
                      <Badge variant="outline">
                        {table.data_source_format}
                      </Badge>
                    ) : null}
                  </CardDescription>
                </CardHeader>
                <CardContent className="text-xs text-muted-foreground">
                  {table.columns?.length ?? 0} column
                  {(table.columns?.length ?? 0) === 1 ? "" : "s"}
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      </section>
    </div>
  )
}
