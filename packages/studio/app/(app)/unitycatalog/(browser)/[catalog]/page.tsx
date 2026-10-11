"use client"

import { IconDatabase, IconFolder } from "@tabler/icons-react"
import Link from "next/link"
import { use } from "react"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useBrowser } from "../browser-context"
import { HeaderBlock, MetaGrid, NotFound } from "../detail"
import { formatDate } from "../../types"

export default function CatalogPage({
  params,
}: {
  params: Promise<{ catalog: string }>
}) {
  const { catalog: catalogParam } = use(params)
  const { catalogs, isPending } = useBrowser()
  const catalog = catalogs.find((c) => c.name === catalogParam)

  if (!catalog) {
    return (
      <NotFound
        message={isPending ? "Loading…" : `Catalog "${catalogParam}" not found.`}
      />
    )
  }

  return (
    <div className="space-y-6">
      <HeaderBlock
        icon={<IconDatabase className="size-4" />}
        kind="Catalog"
        title={catalog.name}
        subtitle={catalog.comment ?? undefined}
      />
      <MetaGrid
        items={[
          ["Owner", catalog.owner ?? "—"],
          ["ID", catalog.id ?? "—"],
          ["Storage root", catalog.storage_root ?? "—"],
          ["Created", formatDate(catalog.created_at)],
          ["Updated", formatDate(catalog.updated_at)],
          ["Schemas", String(catalog.schemas?.length ?? 0)],
        ]}
      />
      <section>
        <h3 className="mb-2 text-xs font-semibold">Schemas</h3>
        <div className="grid gap-2 sm:grid-cols-2">
          {(catalog.schemas ?? []).map((schema) => (
            <Link
              key={schema.name}
              href={`/unitycatalog/${encodeURIComponent(catalog.name)}/${encodeURIComponent(schema.name)}`}
            >
              <Card size="sm" className="hover:bg-muted/40">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-xs">
                    <IconFolder className="size-3.5 text-muted-foreground" />
                    {schema.full_name ?? `${catalog.name}.${schema.name}`}
                  </CardTitle>
                  {schema.comment ? (
                    <CardDescription>{schema.comment}</CardDescription>
                  ) : null}
                </CardHeader>
                <CardContent className="text-xs text-muted-foreground">
                  {schema.tables?.length ?? 0} table
                  {(schema.tables?.length ?? 0) === 1 ? "" : "s"}
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
      </section>
    </div>
  )
}
