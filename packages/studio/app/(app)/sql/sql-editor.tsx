"use client"

import {
  IconAlertCircle,
  IconDatabase,
  IconPlayerPlay,
} from "@tabler/icons-react"
import { type FormEvent, useState, useTransition } from "react"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Textarea } from "@/components/ui/textarea"
import { executeSql, type SqlQueryResult } from "@/app/(app)/sql/actions"

const DEFAULT_SQL = "SELECT * FROM hello ORDER BY id;"

export function SqlEditor() {
  const [sql, setSql] = useState(DEFAULT_SQL)
  const [result, setResult] = useState<SqlQueryResult | null>(null)
  const [isPending, startTransition] = useTransition()

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    startTransition(async () => setResult(await executeSql(sql)))
  }

  return (
    <main className="flex min-h-dvh flex-col gap-4 p-6">
      <header>
        <h1 className="flex items-center gap-2 text-lg font-semibold">
          <IconDatabase className="size-5" />
          DuckDB SQL
        </h1>
        <p className="text-xs text-muted-foreground">
          Execute SQL on the Mizuumi Duck server.
        </p>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>Query</CardTitle>
          <CardDescription>
            Press Command/Control + Enter to execute.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-3" onSubmit={submit}>
            <Textarea
              aria-label="SQL query"
              className="min-h-48 resize-y font-mono"
              disabled={isPending}
              spellCheck={false}
              value={sql}
              onChange={(event) => setSql(event.target.value)}
              onKeyDown={(event) => {
                if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                  event.currentTarget.form?.requestSubmit()
                }
              }}
            />
            <Button type="submit" disabled={isPending || !sql.trim()}>
              <IconPlayerPlay />
              {isPending ? "Executing…" : "Execute"}
            </Button>
          </form>
        </CardContent>
      </Card>

      {result ? <QueryResult result={result} /> : null}
    </main>
  )
}

function QueryResult({ result }: { result: SqlQueryResult }) {
  if (result.error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-destructive">
            <IconAlertCircle className="size-4" />
            Query failed
          </CardTitle>
          <CardDescription className="whitespace-pre-wrap break-words font-mono">
            {result.error}
          </CardDescription>
        </CardHeader>
      </Card>
    )
  }

  const columns = result.columns ?? []
  const rows = result.rows ?? []
  const keyedColumns = addKeys(columns)
  const keyedRows = addKeys(rows)

  return (
    <Card className="min-h-0 flex-1 overflow-hidden">
      <CardHeader>
        <CardTitle>Result</CardTitle>
        <CardDescription>
          {rows.length.toLocaleString()} row{rows.length === 1 ? "" : "s"} in{" "}
          {result.elapsedMs ?? 0} ms
        </CardDescription>
      </CardHeader>
      <CardContent className="min-h-0 overflow-auto">
        <table className="w-full border-collapse text-left text-xs">
          <thead className="sticky top-0 bg-card">
            <tr>
              {keyedColumns.map(({ item: column, key }, index) => (
                <th
                  className="border-b px-3 py-2 font-medium whitespace-nowrap"
                  key={key}
                >
                  <span>{column}</span>
                  <span className="ml-2 font-normal text-muted-foreground">
                    {result.columnTypes?.[index]}
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {keyedRows.map(({ item: row, key }) => (
              <tr className="border-b last:border-0" key={key}>
                {row.map((value, columnIndex) => (
                  <td
                    className="max-w-96 px-3 py-2 align-top whitespace-pre-wrap"
                    key={keyedColumns[columnIndex]?.key}
                  >
                    {formatValue(value)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </CardContent>
    </Card>
  )
}

function formatValue(value: unknown) {
  if (value === null) return "NULL"
  if (typeof value === "object") return JSON.stringify(value)
  return String(value)
}

function addKeys<T>(items: T[]) {
  const occurrences = new Map<string, number>()

  return items.map((item) => {
    const value = JSON.stringify(item) ?? String(item)
    const occurrence = occurrences.get(value) ?? 0
    occurrences.set(value, occurrence + 1)
    return { item, key: `${value}:${occurrence}` }
  })
}
