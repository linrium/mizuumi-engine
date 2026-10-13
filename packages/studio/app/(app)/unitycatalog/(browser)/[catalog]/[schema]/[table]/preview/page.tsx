"use client"

import { IconAlertCircle, IconRefresh } from "@tabler/icons-react"
import {
  AllCommunityModule,
  type ColDef,
  themeQuartz,
  type ValueFormatterParams,
} from "ag-grid-community"
import { AgGridProvider, AgGridReact } from "ag-grid-react"
import { cn } from "cn"
import {
  use,
  useCallback,
  useEffect,
  useMemo,
  useState,
  useTransition,
} from "react"
import { executeSql, type SqlQueryResult } from "@/app/(app)/sql/actions"
import { Button } from "@/components/ui/button"

const PREVIEW_LIMIT = 100
const modules = [AllCommunityModule]

type Row = unknown[]

const previewTheme = themeQuartz.withParams({
  accentColor: "var(--primary)",
  backgroundColor: "var(--background)",
  borderColor: "var(--border)",
  fontFamily: "var(--font-sans)",
  fontSize: 12,
  foregroundColor: "var(--foreground)",
  headerBackgroundColor: "var(--muted)",
  headerFontSize: 12,
  headerFontWeight: 500,
  headerHeight: 36,
  rowHeight: 32,
  wrapperBorder: false,
})

const defaultColDef: ColDef<Row> = {
  filter: true,
  maxWidth: 360,
  minWidth: 120,
  resizable: true,
  sortable: true,
}

export default function TablePreviewPage({
  params,
}: {
  params: Promise<{ catalog: string; schema: string; table: string }>
}) {
  const { catalog, schema, table } = use(params)
  const [result, setResult] = useState<SqlQueryResult | null>(null)
  const [isPending, startTransition] = useTransition()

  const load = useCallback(() => {
    const sql = `SELECT * FROM ${quoteIdent(catalog)}.${quoteIdent(schema)}.${quoteIdent(table)} LIMIT ${PREVIEW_LIMIT}`
    startTransition(async () => setResult(await executeSql(sql)))
  }, [catalog, schema, table])

  useEffect(() => {
    load()
  }, [load])

  const columnDefs = useMemo<ColDef<Row>[]>(() => {
    const names = result?.columns ?? []
    const types = result?.columnTypes ?? []

    return names.map((name, index) => ({
      colId: `${name}:${index}`,
      headerName: name,
      headerTooltip: types[index] ? `${name} · ${types[index]}` : name,
      tooltipValueGetter: ({ value }) => formatValue(value),
      valueFormatter: (params: ValueFormatterParams<Row>) =>
        formatValue(params.value),
      valueGetter: ({ data }) => data?.[index],
    }))
  }, [result?.columns, result?.columnTypes])

  const rowData = useMemo<Row[]>(() => result?.rows ?? [], [result?.rows])

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-xs text-muted-foreground">
          {result && !result.error ? (
            <>
              {rowData.length.toLocaleString()} row
              {rowData.length === 1 ? "" : "s"} · {result.elapsedMs ?? 0} ms ·
              limit {PREVIEW_LIMIT}
            </>
          ) : null}
        </div>
        <Button size="sm" variant="outline" disabled={isPending} onClick={load}>
          <IconRefresh className={cn(isPending && "animate-spin")} />
          {isPending ? "Loading" : "Refresh"}
        </Button>
      </div>

      {result?.error ? (
        <div className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          <IconAlertCircle className="mt-0.5 size-4 shrink-0" />
          <div className="min-w-0 whitespace-pre-wrap break-words font-mono">
            {result.error}
          </div>
        </div>
      ) : (
        <div className="h-[min(60vh,32rem)] min-h-72 overflow-hidden rounded-md ring-1 ring-foreground/10">
          <AgGridProvider modules={modules}>
            <AgGridReact<Row>
              autoSizeStrategy={{ type: "fitCellContents" }}
              columnDefs={columnDefs}
              defaultColDef={defaultColDef}
              loading={isPending || result === null}
              rowData={rowData}
              theme={previewTheme}
              tooltipShowDelay={150}
            />
          </AgGridProvider>
        </div>
      )}
    </div>
  )
}

function quoteIdent(name: string) {
  return `"${name.replace(/"/g, '""')}"`
}

function formatValue(value: unknown) {
  if (value === null || value === undefined) return "NULL"
  if (typeof value === "object") return JSON.stringify(value)
  return String(value)
}
