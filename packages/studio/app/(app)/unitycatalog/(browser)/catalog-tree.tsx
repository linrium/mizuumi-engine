"use client"

import {
  IconChevronDown,
  IconChevronRight,
  IconDatabase,
  IconFolder,
  IconTable,
} from "@tabler/icons-react"
import { cn } from "cn"
import Link from "next/link"
import { useParams } from "next/navigation"
import { useState } from "react"
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { useBrowser } from "./browser-context"

export function CatalogTree() {
  const { catalogs, isPending } = useBrowser()
  const params = useParams<{
    catalog?: string
    schema?: string
    table?: string
  }>()
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})

  const toggle = (key: string) =>
    setExpanded((prev) => ({ ...prev, [key]: !prev[key] }))

  return (
    <Card className="min-h-0 overflow-hidden p-0">
      <CardHeader className="border-b py-3">
        <CardTitle className="text-sm">Browser</CardTitle>
        <CardDescription>
          {catalogs.length} catalog{catalogs.length === 1 ? "" : "s"}
        </CardDescription>
      </CardHeader>
      <ScrollArea className="h-full">
        <div className="p-2">
          {isPending && catalogs.length === 0 ? (
            <div className="px-2 py-6 text-center text-xs text-muted-foreground">
              Loading Unity Catalog metadata…
            </div>
          ) : catalogs.length === 0 ? (
            <div className="px-2 py-6 text-center text-xs text-muted-foreground">
              No catalogs found.
            </div>
          ) : (
            <ul className="space-y-0.5">
              {catalogs.map((catalog) => {
                const cKey = `c:${catalog.name}`
                const cOpen =
                  expanded[cKey] ??
                  (params.catalog === catalog.name)
                const cSel =
                  params.catalog === catalog.name && !params.schema
                return (
                  <li key={catalog.name}>
                    <TreeRow
                      icon={<IconDatabase className="size-3.5" />}
                      label={catalog.name}
                      href={`/unitycatalog/${encodeURIComponent(catalog.name)}`}
                      selected={cSel}
                      open={cOpen}
                      hasChildren={(catalog.schemas?.length ?? 0) > 0}
                      onToggle={() => toggle(cKey)}
                    />
                    {cOpen && catalog.schemas?.length ? (
                      <ul className="ml-4 space-y-0.5 border-l pl-2">
                        {catalog.schemas.map((schema) => {
                          const sKey = `s:${catalog.name}.${schema.name}`
                          const sOpen =
                            expanded[sKey] ??
                            (params.catalog === catalog.name &&
                              params.schema === schema.name)
                          const sSel =
                            params.catalog === catalog.name &&
                            params.schema === schema.name &&
                            !params.table
                          return (
                            <li key={schema.name}>
                              <TreeRow
                                icon={<IconFolder className="size-3.5" />}
                                label={schema.name}
                                href={`/unitycatalog/${encodeURIComponent(catalog.name)}/${encodeURIComponent(schema.name)}`}
                                selected={sSel}
                                open={sOpen}
                                hasChildren={(schema.tables?.length ?? 0) > 0}
                                onToggle={() => toggle(sKey)}
                              />
                              {sOpen && schema.tables?.length ? (
                                <ul className="ml-4 space-y-0.5 border-l pl-2">
                                  {schema.tables.map((table) => {
                                    const tSel =
                                      params.catalog === catalog.name &&
                                      params.schema === schema.name &&
                                      params.table === table.name
                                    return (
                                      <li key={table.name}>
                                        <TreeRow
                                          icon={
                                            <IconTable className="size-3.5" />
                                          }
                                          label={table.name}
                                          href={`/unitycatalog/${encodeURIComponent(catalog.name)}/${encodeURIComponent(schema.name)}/${encodeURIComponent(table.name)}`}
                                          selected={tSel}
                                          hasChildren={false}
                                        />
                                      </li>
                                    )
                                  })}
                                </ul>
                              ) : null}
                            </li>
                          )
                        })}
                      </ul>
                    ) : null}
                  </li>
                )
              })}
            </ul>
          )}
        </div>
      </ScrollArea>
    </Card>
  )
}

function TreeRow({
  icon,
  label,
  href,
  selected,
  open,
  hasChildren,
  onToggle,
}: {
  icon: React.ReactNode
  label: string
  href: string
  selected?: boolean
  open?: boolean
  hasChildren?: boolean
  onToggle?: () => void
}) {
  return (
    <div
      className={cn(
        "group flex items-center gap-1 rounded px-1.5 py-1 text-xs hover:bg-muted",
        selected && "bg-muted font-medium",
      )}
    >
      <button
        type="button"
        onClick={onToggle}
        className={cn(
          "flex size-4 shrink-0 items-center justify-center rounded text-muted-foreground",
          !hasChildren && "invisible",
        )}
        aria-label={open ? "Collapse" : "Expand"}
      >
        {open ? (
          <IconChevronDown className="size-3" />
        ) : (
          <IconChevronRight className="size-3" />
        )}
      </button>
      <Link
        href={href}
        className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
      >
        <span className="text-muted-foreground">{icon}</span>
        <span className="truncate">{label}</span>
      </Link>
    </div>
  )
}
