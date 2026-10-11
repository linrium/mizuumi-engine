"use client"

import { cn } from "cn"
import { use } from "react"
import { useBrowser } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import { MetaGrid, NotFound } from "@/app/(app)/unitycatalog/(browser)/detail"
import { formatDate } from "@/app/(app)/unitycatalog/types"

export default function TableOverviewPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
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
    return <NotFound message={isPending ? "Loading…" : "Table not found."} />
  }

  const columns = table.columns ?? []
  const properties = Object.entries(table.properties ?? {})

  return (
    <div className="space-y-6">
      <MetaGrid
        items={[
          ["Owner", table.owner ?? "—"],
          ["Table ID", table.table_id ?? "—"],
          ["Created", formatDate(table.created_at)],
          ["Updated", formatDate(table.updated_at)],
          ["Storage location", table.storage_location ?? "—"],
          ["Columns", String(columns.length)],
        ]}
      />

      <section>
        <h3 className="mb-2 text-xs font-semibold">Columns</h3>
        <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
          <table className="w-full text-xs">
            <thead className="bg-muted/50 text-muted-foreground">
              <tr>
                <th className="px-3 py-2 text-left font-medium">#</th>
                <th className="px-3 py-2 text-left font-medium">Name</th>
                <th className="px-3 py-2 text-left font-medium">Type</th>
                <th className="px-3 py-2 text-left font-medium">Nullable</th>
                <th className="px-3 py-2 text-left font-medium">Comment</th>
              </tr>
            </thead>
            <tbody>
              {columns.map((col, i) => (
                <tr
                  key={col.name}
                  className={cn("border-t", i % 2 === 1 && "bg-muted/20")}
                >
                  <td className="px-3 py-1.5 text-muted-foreground">
                    {col.position ?? i}
                  </td>
                  <td className="px-3 py-1.5 font-mono">{col.name}</td>
                  <td className="px-3 py-1.5">
                    <code className="rounded bg-muted px-1.5 py-0.5 text-[0.7rem]">
                      {col.type_text ?? col.type_name ?? "—"}
                    </code>
                  </td>
                  <td className="px-3 py-1.5 text-muted-foreground">
                    {col.nullable ? "yes" : "no"}
                  </td>
                  <td className="px-3 py-1.5 text-muted-foreground">
                    {col.comment ?? "—"}
                  </td>
                </tr>
              ))}
              {columns.length === 0 ? (
                <tr>
                  <td
                    colSpan={5}
                    className="px-3 py-6 text-center text-muted-foreground"
                  >
                    No columns.
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </section>

      {properties.length ? (
        <section>
          <h3 className="mb-2 text-xs font-semibold">Properties</h3>
          <div className="rounded-md ring-1 ring-foreground/10">
            <dl className="divide-y text-xs">
              {properties.map(([key, value]) => (
                <div
                  key={key}
                  className="grid grid-cols-[minmax(0,1fr)_minmax(0,2fr)] gap-3 px-3 py-1.5"
                >
                  <dt className="truncate font-mono text-muted-foreground">
                    {key}
                  </dt>
                  <dd className="truncate font-mono">{value}</dd>
                </div>
              ))}
            </dl>
          </div>
        </section>
      ) : null}
    </div>
  )
}
