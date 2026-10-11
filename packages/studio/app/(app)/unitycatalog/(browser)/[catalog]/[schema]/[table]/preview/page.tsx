"use client"

import { IconAlertCircle, IconRefresh } from "@tabler/icons-react"
import {
  columnVisibilityFeature,
  createColumnHelper,
  tableFeatures,
  useTable,
} from "@tanstack/react-table"
import { cn } from "cn"
import { use, useCallback, useEffect, useMemo, useState, useTransition } from "react"
import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { executeSql, type SqlQueryResult } from "@/app/(app)/sql/actions"

const PREVIEW_LIMIT = 100

const features = tableFeatures({ columnVisibilityFeature })

type Row = Record<string, unknown>

const columnHelper = createColumnHelper<typeof features, Row>()

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

  const columns = useMemo(() => {
    const names: string[] = result?.columns ?? []
    const types: string[] = result?.columnTypes ?? []
    return names.map((name, i) =>
      columnHelper.accessor((row: Row) => row[name], {
        id: `${name}:${i}`,
        header: () => (
          <div className="flex items-baseline gap-2">
            <span>{name}</span>
            <span className="text-[10px] font-normal text-muted-foreground">
              {types[i]}
            </span>
          </div>
        ),
        cell: ({ getValue }) => (
          <span className="truncate font-mono">{formatValue(getValue())}</span>
        ),
      }),
    )
  }, [result?.columns, result?.columnTypes])

  const data = useMemo<Row[]>(() => {
    const names: string[] = result?.columns ?? []
    const rows: unknown[][] = result?.rows ?? []
    return rows.map((cells: unknown[]) =>
      Object.fromEntries(names.map((n: string, i: number) => [n, cells[i]])),
    )
  }, [result?.columns, result?.rows])

  const tableInstance = useTable({ features, data, columns })

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-xs text-muted-foreground">
          {result && !result.error ? (
            <>
              {data.length.toLocaleString()} row{data.length === 1 ? "" : "s"}{" "}
              · {result.elapsedMs ?? 0} ms · limit {PREVIEW_LIMIT}
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
        <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
          <Table>
            <TableHeader className="bg-muted/50">
              {tableInstance.getHeaderGroups().map((group) => (
                <TableRow key={group.id}>
                  {group.headers.map((header) => (
                    <TableHead key={header.id}>
                      <tableInstance.FlexRender header={header} />
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {tableInstance.getRowModel().rows.map((row) => (
                <TableRow key={row.id}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id} className="max-w-96">
                      <tableInstance.FlexRender cell={cell} />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
              {!isPending && data.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={columns.length || 1}
                    className="h-20 text-center text-muted-foreground"
                  >
                    No rows.
                  </TableCell>
                </TableRow>
              ) : null}
              {isPending && data.length === 0 ? (
                <TableRow>
                  <TableCell
                    colSpan={columns.length || 1}
                    className="h-20 text-center text-muted-foreground"
                  >
                    Loading preview…
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
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
