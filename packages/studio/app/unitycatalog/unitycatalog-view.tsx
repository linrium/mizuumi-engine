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
import { cn } from "cn"
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
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Separator } from "@/components/ui/separator"
import { authClient } from "@/lib/auth-client"
import {
  getUnityCatalogAllAssignments,
  getUnityCatalogInfo,
  getUnityCatalogPermissions,
  type PrivilegeAssignment,
  revokeAllPrivileges,
  type SecurableAssignments,
  type SecurableType,
  updateUnityCatalogPermissions,
} from "./actions"

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

type Tab = "browser" | "permissions" | "assignments"

export function UnityCatalogView() {
  const [tab, setTab] = useState<Tab>("browser")
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
    if (selection.kind === "catalog")
      return { kind: "catalog" as const, catalog }
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
        {tab === "browser" ? (
          <Button
            size="sm"
            variant="outline"
            disabled={isPending}
            onClick={load}
          >
            <IconRefresh className={cn(isPending && "animate-spin")} />
            {isPending ? "Loading" : "Refresh"}
          </Button>
        ) : null}
      </header>

      <div className="flex gap-1 border-b">
        {(
          [
            ["browser", "Browser"],
            ["permissions", "Permissions"],
            ["assignments", "Assignments"],
          ] as const
        ).map(([value, label]) => (
          <button
            key={value}
            type="button"
            onClick={() => setTab(value)}
            className={cn(
              "border-b-2 px-3 py-1.5 text-xs font-medium transition-colors",
              tab === value
                ? "border-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground",
            )}
          >
            {label}
          </button>
        ))}
      </div>

      {tab === "assignments" ? (
        <AssignmentsPanel />
      ) : tab === "permissions" ? (
        <PermissionsPanel catalogs={catalogs} />
      ) : (
        <BrowserPanel
          catalogs={catalogs}
          data={data}
          error={error}
          expanded={expanded}
          isPending={isPending}
          selected={selected}
          selection={selection}
          setSelection={setSelection}
          toggle={toggle}
        />
      )}
    </main>
  )
}

function BrowserPanel({
  catalogs,
  data,
  error,
  expanded,
  isPending,
  selected,
  selection,
  setSelection,
  toggle,
}: {
  catalogs: Catalog[]
  data: UnityCatalogData | null
  error: string | null
  expanded: Record<string, boolean>
  isPending: boolean
  selected:
    | { kind: "catalog"; catalog: Catalog }
    | { kind: "schema"; catalog: Catalog; schema: Schema }
    | { kind: "table"; catalog: Catalog; schema: Schema; table: Table }
    | null
  selection: Selection
  setSelection: (s: Selection) => void
  toggle: (key: string) => void
}) {
  return (
    <>
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
                                    hasChildren={
                                      (schema.tables?.length ?? 0) > 0
                                    }
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
    </>
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

const PRIVILEGES_BY_TYPE: Record<SecurableType, string[]> = {
  catalog: [
    "USE CATALOG",
    "CREATE SCHEMA",
    "USE SCHEMA",
    "CREATE TABLE",
    "SELECT",
    "MODIFY",
  ],
  schema: ["USE SCHEMA", "CREATE TABLE", "SELECT", "MODIFY"],
  table: ["SELECT", "MODIFY"],
}

function PermissionsPanel({ catalogs }: { catalogs: Catalog[] }) {
  const [securableType, setSecurableType] = useState<SecurableType>("catalog")
  const [fullName, setFullName] = useState<string>("")
  const [principal, setPrincipal] = useState<string>("")
  const [checked, setChecked] = useState<Record<string, boolean>>({})
  const [assignments, setAssignments] = useState<PrivilegeAssignment[]>([])
  const [message, setMessage] = useState<
    { kind: "error"; text: string } | { kind: "success"; text: string } | null
  >(null)
  const [isPending, startTransition] = useTransition()

  const availableTargets = useMemo(() => {
    const items: Array<{ value: string; label: string }> = []
    for (const c of catalogs) {
      if (securableType === "catalog") {
        items.push({ value: c.name, label: c.name })
      } else if (securableType === "schema") {
        for (const s of c.schemas ?? []) {
          const full = s.full_name ?? `${c.name}.${s.name}`
          items.push({ value: full, label: full })
        }
      } else {
        for (const s of c.schemas ?? []) {
          for (const t of s.tables ?? []) {
            const full = `${c.name}.${s.name}.${t.name}`
            items.push({ value: full, label: full })
          }
        }
      }
    }
    return items
  }, [catalogs, securableType])

  const privileges = PRIVILEGES_BY_TYPE[securableType]

  const togglePrivilege = (p: string) =>
    setChecked((prev) => ({ ...prev, [p]: !prev[p] }))

  const selectedPrivileges = privileges.filter((p) => checked[p])

  const loadAssignments = useCallback(() => {
    if (!fullName) return
    startTransition(async () => {
      const res = await getUnityCatalogPermissions(securableType, fullName)
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog",
        })
        return
      }
      if (res.error) {
        setAssignments([])
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage(null)
    })
  }, [fullName, securableType])

  const submit = (action: "grant" | "revoke") => {
    if (!fullName || !principal || selectedPrivileges.length === 0) {
      setMessage({
        kind: "error",
        text: "Select a securable, principal, and at least one privilege.",
      })
      return
    }
    startTransition(async () => {
      const res = await updateUnityCatalogPermissions({
        securableType,
        fullName,
        principal,
        add: action === "grant" ? selectedPrivileges : [],
        remove: action === "revoke" ? selectedPrivileges : [],
      })
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog",
        })
        return
      }
      if (res.error) {
        setMessage({ kind: "error", text: res.error })
        return
      }
      setAssignments(res.data?.privilege_assignments ?? [])
      setMessage({
        kind: "success",
        text: `${action === "grant" ? "Granted" : "Revoked"} ${selectedPrivileges.join(", ")} ${action === "grant" ? "to" : "from"} ${principal}.`,
      })
      setChecked({})
    })
  }

  return (
    <div className="grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[1fr_1fr]">
      <Card className="min-h-0 overflow-hidden p-0">
        <CardHeader className="border-b py-3">
          <CardTitle className="text-sm">Grant permissions</CardTitle>
          <CardDescription>
            Grant or revoke Unity Catalog privileges on a securable for a
            principal.
          </CardDescription>
        </CardHeader>
        <ScrollArea className="h-full">
          <CardContent className="space-y-4 p-6">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-xs font-medium" htmlFor="securable-type">
                  Securable type
                </label>
                <Select
                  value={securableType}
                  onValueChange={(v) => {
                    if (!v) return
                    setSecurableType(v as SecurableType)
                    setFullName("")
                    setChecked({})
                  }}
                >
                  <SelectTrigger className="h-9 w-full" id="securable-type">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="catalog">Catalog</SelectItem>
                      <SelectItem value="schema">Schema</SelectItem>
                      <SelectItem value="table">Table</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <label className="text-xs font-medium" htmlFor="full-name">
                  Securable
                </label>
                {availableTargets.length > 0 ? (
                  <Select
                    value={fullName}
                    onValueChange={(v) => v && setFullName(v)}
                  >
                    <SelectTrigger className="h-9 w-full" id="full-name">
                      <SelectValue placeholder="Select…" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        {availableTargets.map((o) => (
                          <SelectItem key={o.value} value={o.value}>
                            {o.label}
                          </SelectItem>
                        ))}
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                ) : (
                  <Input
                    id="full-name"
                    placeholder={
                      securableType === "catalog"
                        ? "catalog_name"
                        : securableType === "schema"
                          ? "catalog.schema"
                          : "catalog.schema.table"
                    }
                    value={fullName}
                    onChange={(e) => setFullName(e.target.value)}
                  />
                )}
              </div>
            </div>

            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="principal">
                Principal (user, service principal, or group)
              </label>
              <Input
                id="principal"
                placeholder="user@example.com"
                value={principal}
                onChange={(e) => setPrincipal(e.target.value)}
              />
            </div>

            <div className="space-y-1.5">
              <div className="text-xs font-medium">Privileges</div>
              <div className="grid grid-cols-2 gap-1.5">
                {privileges.map((p) => (
                  <label
                    key={p}
                    className="flex items-center gap-2 rounded border px-2 py-1.5 text-xs hover:bg-muted"
                  >
                    <input
                      type="checkbox"
                      className="size-3.5"
                      checked={!!checked[p]}
                      onChange={() => togglePrivilege(p)}
                    />
                    <span className="font-mono">{p}</span>
                  </label>
                ))}
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              <Button
                size="sm"
                disabled={isPending}
                onClick={() => submit("grant")}
              >
                Grant
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={isPending}
                onClick={() => submit("revoke")}
              >
                Revoke
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={isPending || !fullName}
                onClick={loadAssignments}
              >
                <IconRefresh className={cn(isPending && "animate-spin")} />
                Load current
              </Button>
            </div>

            {message ? (
              <div
                className={cn(
                  "rounded border px-3 py-2 text-xs",
                  message.kind === "error"
                    ? "border-destructive/40 text-destructive"
                    : "border-emerald-500/40 text-emerald-700 dark:text-emerald-400",
                )}
              >
                {message.text}
              </div>
            ) : null}
          </CardContent>
        </ScrollArea>
      </Card>

      <Card className="min-h-0 overflow-hidden p-0">
        <CardHeader className="border-b py-3">
          <CardTitle className="text-sm">Current assignments</CardTitle>
          <CardDescription>
            {fullName
              ? `${securableType}: ${fullName}`
              : "No securable selected"}
          </CardDescription>
        </CardHeader>
        <ScrollArea className="h-full">
          <div className="p-6">
            {assignments.length === 0 ? (
              <div className="py-6 text-center text-xs text-muted-foreground">
                No assignments loaded. Pick a securable and click "Load
                current".
              </div>
            ) : (
              <ul className="space-y-2">
                {assignments.map((a) => (
                  <li
                    key={a.principal}
                    className="rounded-md ring-1 ring-foreground/10 p-3"
                  >
                    <div className="font-mono text-xs">{a.principal}</div>
                    <div className="mt-1.5 flex flex-wrap gap-1">
                      {a.privileges.map((p) => (
                        <Badge key={p} variant="secondary">
                          {p}
                        </Badge>
                      ))}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </ScrollArea>
      </Card>
    </div>
  )
}

function AssignmentsPanel() {
  const [assignments, setAssignments] = useState<SecurableAssignments[]>([])
  const [error, setError] = useState<string | null>(null)
  const [loaded, setLoaded] = useState(false)
  const [filter, setFilter] = useState("")
  const [isPending, startTransition] = useTransition()
  const [deletingKey, setDeletingKey] = useState<string | null>(null)

  const load = useCallback(() => {
    startTransition(async () => {
      const res = await getUnityCatalogAllAssignments()
      if (res.needsReconnect) {
        await authClient.signIn.social({
          provider: "keycloak",
          callbackURL: "/unitycatalog",
        })
        return
      }
      if (res.error) {
        setError(res.error)
        setAssignments([])
        setLoaded(true)
        return
      }
      setError(null)
      setAssignments(res.data?.assignments ?? [])
      setLoaded(true)
    })
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const removeAssignment = useCallback(
    (securableType: string, fullName: string, principal: string) => {
      const group = assignments.find(
        (a) => a.securable_type === securableType && a.full_name === fullName,
      )
      const pa = group?.privilege_assignments.find(
        (p) => p.principal === principal,
      )
      if (!pa || pa.privileges.length === 0) return
      if (
        !window.confirm(
          `Revoke ${pa.privileges.join(", ")} from ${principal} on ${securableType} ${fullName}?`,
        )
      )
        return
      const key = `${securableType}:${fullName}:${principal}`
      setDeletingKey(key)
      startTransition(async () => {
        const res = await revokeAllPrivileges({
          securableType,
          fullName,
          principal,
          privileges: pa.privileges,
        })
        setDeletingKey(null)
        if (res.needsReconnect) {
          await authClient.signIn.social({
            provider: "keycloak",
            callbackURL: "/unitycatalog",
          })
          return
        }
        if (res.error) {
          setError(res.error)
          return
        }
        setError(null)
        setAssignments((prev) =>
          prev
            .map((group) => {
              if (
                group.securable_type !== securableType ||
                group.full_name !== fullName
              )
                return group
              return {
                ...group,
                privilege_assignments: group.privilege_assignments.filter(
                  (p) => p.principal !== principal,
                ),
              }
            })
            .filter((group) => group.privilege_assignments.length > 0),
        )
      })
    },
    [assignments],
  )

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase()
    if (!q) return assignments
    return assignments.filter((a) => {
      if (
        a.full_name.toLowerCase().includes(q) ||
        a.securable_type.toLowerCase().includes(q)
      )
        return true
      return a.privilege_assignments.some(
        (pa) =>
          pa.principal.toLowerCase().includes(q) ||
          pa.privileges.some((p) => p.toLowerCase().includes(q)),
      )
    })
  }, [assignments, filter])

  const totalPrincipals = assignments.reduce(
    (n, a) => n + a.privilege_assignments.length,
    0,
  )

  return (
    <Card className="min-h-0 flex-1 overflow-hidden p-0">
      <CardHeader className="flex flex-row items-start justify-between gap-3 border-b py-3">
        <div>
          <CardTitle className="text-sm">All assignments</CardTitle>
          <CardDescription>
            {assignments.length} securable
            {assignments.length === 1 ? "" : "s"} · {totalPrincipals} principal
            {totalPrincipals === 1 ? "" : "s"}
          </CardDescription>
        </div>
        <div className="flex items-center gap-2">
          <Input
            placeholder="Filter by name, principal, or privilege"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="h-8 w-72"
          />
          <Button
            size="sm"
            variant="outline"
            disabled={isPending}
            onClick={load}
          >
            <IconRefresh className={cn(isPending && "animate-spin")} />
            {isPending ? "Loading" : "Refresh"}
          </Button>
        </div>
      </CardHeader>
      <ScrollArea className="h-full">
        <div className="p-6">
          {error ? (
            <div className="rounded border border-destructive/40 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          ) : !loaded && isPending ? (
            <div className="py-10 text-center text-xs text-muted-foreground">
              Loading assignments…
            </div>
          ) : filtered.length === 0 ? (
            <div className="py-10 text-center text-xs text-muted-foreground">
              {assignments.length === 0
                ? "No permissions assigned."
                : "No assignments match the filter."}
            </div>
          ) : (
            <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
              <table className="w-full text-xs">
                <thead className="bg-muted/50 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">Type</th>
                    <th className="px-3 py-2 text-left font-medium">
                      Securable
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      Principal
                    </th>
                    <th className="px-3 py-2 text-left font-medium">
                      Privileges
                    </th>
                    <th className="px-3 py-2 text-right font-medium">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {filtered.flatMap((group) =>
                    group.privilege_assignments.map((pa, i) => (
                      <tr
                        key={`${group.securable_type}:${group.full_name}:${pa.principal}`}
                        className={cn("border-t", i % 2 === 1 && "bg-muted/20")}
                      >
                        <td className="px-3 py-1.5">
                          <Badge variant="outline">
                            {group.securable_type}
                          </Badge>
                        </td>
                        <td className="px-3 py-1.5 font-mono break-all">
                          {group.full_name}
                        </td>
                        <td className="px-3 py-1.5 font-mono break-all">
                          {pa.principal}
                        </td>
                        <td className="px-3 py-1.5">
                          <div className="flex flex-wrap gap-1">
                            {pa.privileges.map((p) => (
                              <Badge key={p} variant="secondary">
                                {p}
                              </Badge>
                            ))}
                          </div>
                        </td>
                        <td className="px-3 py-1.5 text-right">
                          <Button
                            size="sm"
                            variant="ghost"
                            disabled={
                              deletingKey ===
                              `${group.securable_type}:${group.full_name}:${pa.principal}`
                            }
                            onClick={() =>
                              removeAssignment(
                                group.securable_type,
                                group.full_name,
                                pa.principal,
                              )
                            }
                          >
                            {deletingKey ===
                            `${group.securable_type}:${group.full_name}:${pa.principal}`
                              ? "Deleting…"
                              : "Delete"}
                          </Button>
                        </td>
                      </tr>
                    )),
                  )}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </ScrollArea>
    </Card>
  )
}
