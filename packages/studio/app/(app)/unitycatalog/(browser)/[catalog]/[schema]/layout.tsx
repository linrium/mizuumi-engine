"use client"

import { IconFolder } from "@tabler/icons-react"
import { usePathname } from "next/navigation"
import { use } from "react"
import { SchemaNav } from "@/app/(app)/unitycatalog/(browser)/[catalog]/[schema]/schema-nav"
import { useBrowser } from "@/app/(app)/unitycatalog/(browser)/browser-context"
import {
  HeaderBlock,
  NotFound,
} from "@/app/(app)/unitycatalog/(browser)/detail"

export default function SchemaLayout({
  params,
  children,
}: {
  params: Promise<{ catalog: string; schema: string }>
  children: React.ReactNode
}) {
  const { catalog: catalogParam, schema: schemaParam } = use(params)
  const { catalogs, isPending } = useBrowser()
  const catalog = catalogs.find((c) => c.name === catalogParam)
  const schema = catalog?.schemas?.find((s) => s.name === schemaParam)
  const pathname = usePathname()
  const base = `/unitycatalog/${encodeURIComponent(catalogParam)}/${encodeURIComponent(schemaParam)}`
  const isSchemaLevel =
    pathname === base ||
    pathname === `${base}/permissions` ||
    pathname === `${base}/requests`

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

  if (!isSchemaLevel) {
    return <>{children}</>
  }

  return (
    <div className="space-y-3">
      <HeaderBlock
        icon={<IconFolder className="size-4" />}
        kind="Schema"
        title={schema.full_name ?? `${catalog.name}.${schema.name}`}
        subtitle={schema.comment ?? undefined}
      />
      <SchemaNav catalog={catalog.name} schema={schema.name} />
      {children}
    </div>
  )
}
