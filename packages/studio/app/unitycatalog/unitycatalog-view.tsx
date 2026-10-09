"use client"

import {
  IconAlertCircle,
  IconChevronDown,
  IconChevronRight,
  IconDatabase,
  IconFolder,
  IconRefresh,
  IconTable,
} from "@tabler/icons-react"
import { useCallback, useEffect, useMemo, useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { authClient } from "@/lib/auth-client"
import { cn } from "cn"
import { getUnityCatalogInfo } from "./actions"

type Column = {
  name: string
  type_text?: string | null
  type_name?: string | null
  position?: number | null
  nullable?: boolean | null
  comment?: string | null
  partition_index?: number | null
}

type Table = {
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

type Schema = {
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

type Catalog = {
  name: string
  comment?: string | null
  owner?: string | null
  created_at?: number | null
  updated_at?: number | null
  id?: string | null
  storage_root?: string | null
  schemas?: Schema[]
}

type UnityCatalogData = { catalogs?: Catalog[] }

type Selection =
  | { kind: "catalog"; catalog: string }
  | { kind: "schema"; catalog: string; schema: string }
  | { kind: "table"; catalog: string; schema: string; table: string }
  | null

function formatDate(ms?: number | null) {
  if (!ms) return "—"
  return new Date(ms).toLocaleString()
}

export function UnityCatalogView() {
  const [data, setData] = useState<UnityCatalogData | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [selection, setSelection] = useState<Selection>(null)
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})
  const [isPending, startTransition] = useTransition()

  const load = useCallback(() => {
    startTransition(async () => {
      const next = await getUnityCatalogInfo()
      if (next.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog",
        })
        return
      }
      if (next.error) {
        setError(next.error)
        setData(null)
        return
      }
      setError(null)
      setData((next.data as UnityCatalogData | undefined) ?? { catalogs: [] })
    })
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const catalogs = data?.catalogs ?? []

  const selected = useMemo(() => {
    if (!selection) return null
    const catalog = catalogs.find((c) => c.name === selection.catalog)
    if (!catalog) return null
    if (selection.kind === "catalog") return { kind: "catalog" as const, catalog }
    const schema = catalog.schemas?.find((s) => s.name === selection.schema)
    if (!schema) return null
    if (selection.kind === "schema")
      return { kind: "schema" as const, catalog, schema }
    const table = schema.tables?.find((t) => t.name === selection.table)
    if (!table) return null
    return { kind: "table" as const, catalog, schema, table }
  }, [catalogs, selection])

  const toggle = (key: string) =>
    setExpanded((prev) => ({ ...prev, [key]: !prev[key] }))

  return (
    <main className="flex h-dvh flex-col gap-4 p-6">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold">Unity Catalog</h1>
          <p className="text-xs text-muted-foreground">
            Browse catalogs, schemas, and tables.
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          disabled={isPending}
          onClick={load}
        >
          <IconRefresh className={cn(isPending && "animate-spin")} />
          {isPending ? "Loading" : "Refresh"}
        </Button>
      </header>

      {error ? (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <IconAlertCircle className="size-4" />
              Unable to load Unity Catalog
            </CardTitle>
            <CardDescription className="break-all">{error}</CardDescription>
          </CardHeader>
        </Card>
      ) : null}

      <div className="grid min-h-0 flex-1 grid-cols-[320px_1fr] gap-4">
        <Card className="min-h-0 overflow-hidden p-0">
          <CardHeader className="border-b py-3">
            <CardTitle className="text-sm">Browser</CardTitle>
            <CardDescription>
              {catalogs.length} catalog{catalogs.length === 1 ? "" : "s"}
            </CardDescription>
          </CardHeader>
          <ScrollArea className="h-full">
            <div className="p-2">
              {isPending && !data ? (
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
                    const key = `c:${catalog.name}`
                    const open = expanded[key] ?? true
                    const isSel =
                      selection?.kind === "catalog" &&
                      selection.catalog === catalog.name
                    return (
                      <li key={catalog.name}>
                        <TreeRow
                          icon={<IconDatabase className="size-3.5" />}
                          label={catalog.name}
                          selected={isSel}
                          open={open}
                          hasChildren={(catalog.schemas?.length ?? 0) > 0}
                          onToggle={() => toggle(key)}
                          onSelect={() =>
                            setSelection({
                              kind: "catalog",
                              catalog: catalog.name,
                            })
                          }
                        />
                        {open && catalog.schemas?.length ? (
                          <ul className="ml-4 space-y-0.5 border-l pl-2">
                            {catalog.schemas.map((schema) => {
                              const sKey = `s:${catalog.name}.${schema.name}`
                              const sOpen = expanded[sKey] ?? true
                              const sSel =
                                selection?.kind === "schema" &&
                                selection.catalog === catalog.name &&
                                selection.schema === schema.name
                              return (
                                <li key={schema.name}>
                                  <TreeRow
                                    icon={<IconFolder className="size-3.5" />}
                                    label={schema.name}
                                    selected={sSel}
                                    open={sOpen}
                                    hasChildren={(schema.tables?.length ?? 0) > 0}
                                    onToggle={() => toggle(sKey)}
                                    onSelect={() =>
                                      setSelection({
                                        kind: "schema",
                                        catalog: catalog.name,
                                        schema: schema.name,
                                      })
                                    }
                                  />
                                  {sOpen && schema.tables?.length ? (
                                    <ul className="ml-4 space-y-0.5 border-l pl-2">
                                      {schema.tables.map((table) => {
                                        const tSel =
                                          selection?.kind === "table" &&
                                          selection.catalog === catalog.name &&
                                          selection.schema === schema.name &&
                                          selection.table === table.name
                                        return (
                                          <li key={table.name}>
                                            <TreeRow
                                              icon={
                                                <IconTable className="size-3.5" />
                                              }
                                              label={table.name}
                                              selected={tSel}
                                              hasChildren={false}
                                              onSelect={() =>
                                                setSelection({
                                                  kind: "table",
                                                  catalog: catalog.name,
                                                  schema: schema.name,
                                                  table: table.name,
                                                })
                                              }
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

        <Card className="min-h-0 overflow-hidden p-0">
          <ScrollArea className="h-full">
            <div className="p-6">
              <DetailPanel selected={selected} />
            </div>
          </ScrollArea>
        </Card>
      </div>
    </main>
  )
}

function TreeRow({
  icon,
  label,
  selected,
  open,
  hasChildren,
  onToggle,
  onSelect,
}: {
  icon: React.ReactNode
  label: string
  selected?: boolean
  open?: boolean
  hasChildren?: boolean
  onToggle?: () => void
  onSelect?: () => void
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
      <button
        type="button"
        onClick={onSelect}
        className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
      >
        <span className="text-muted-foreground">{icon}</span>
        <span className="truncate">{label}</span>
      </button>
    </div>
  )
}

function DetailPanel({
  selected,
}: {
  selected:
    | { kind: "catalog"; catalog: Catalog }
    | { kind: "schema"; catalog: Catalog; schema: Schema }
    | { kind: "table"; catalog: Catalog; schema: Schema; table: Table }
    | null
}) {
  if (!selected) {
    return (
      <div className="flex h-full min-h-[200px] items-center justify-center text-center text-xs text-muted-foreground">
        Select a catalog, schema, or table on the left to see details.
      </div>
    )
  }

  if (selected.kind === "catalog") {
    const { catalog } = selected
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
              <Card key={schema.name} size="sm">
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
            ))}
          </div>
        </section>
      </div>
    )
  }

  if (selected.kind === "schema") {
    const { catalog, schema } = selected
    return (
      <div className="space-y-6">
        <HeaderBlock
          icon={<IconFolder className="size-4" />}
          kind="Schema"
          title={schema.full_name ?? `${catalog.name}.${schema.name}`}
          subtitle={schema.comment ?? undefined}
        />
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
              <Card key={table.name} size="sm">
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
            ))}
          </div>
        </section>
      </div>
    )
  }

  const { catalog, schema, table } = selected
  const columns = table.columns ?? []
  const properties = Object.entries(table.properties ?? {})

  return (
    <div className="space-y-6">
      <HeaderBlock
        icon={<IconTable className="size-4" />}
        kind="Table"
        title={`${catalog.name}.${schema.name}.${table.name}`}
        subtitle={undefined}
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
                  className={cn(
                    "border-t",
                    i % 2 === 1 && "bg-muted/20",
                  )}
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

function HeaderBlock({
  icon,
  kind,
  title,
  subtitle,
  badges,
}: {
  icon: React.ReactNode
  kind: string
  title: string
  subtitle?: string
  badges?: React.ReactNode
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        {icon}
        <span>{kind}</span>
      </div>
      <h2 className="font-mono text-base font-semibold break-all">{title}</h2>
      {subtitle ? (
        <p className="text-xs text-muted-foreground">{subtitle}</p>
      ) : null}
      {badges ? <div className="flex flex-wrap gap-1.5">{badges}</div> : null}
      <Separator />
    </div>
  )
}

function MetaGrid({ items }: { items: Array<[string, string]> }) {
  return (
    <dl className="grid grid-cols-1 gap-x-6 gap-y-2 text-xs sm:grid-cols-2">
      {items.map(([label, value]) => (
        <div key={label} className="min-w-0">
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="truncate font-mono">{value}</dd>
        </div>
      ))}
    </dl>
  )
}
