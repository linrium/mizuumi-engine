"use client"

import { IconRefresh, IconSearch } from "@tabler/icons-react"
import { useMemo, useState, useTransition } from "react"
import {
  getPipeline,
  listPipelines,
  type Pipeline,
  type PipelineDetail,
} from "@/app/(app)/pipelines/actions"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

type Props = {
  initialPipelines: Pipeline[]
  initialError?: string
}

const STATE_VARIANT: Record<
  string,
  "default" | "secondary" | "destructive" | "outline" | "ghost"
> = {
  RUNNING: "default",
  SUBMITTED: "secondary",
  PENDING_SUBMISSION: "secondary",
  SUBMISSION_FAILED: "destructive",
  COMPLETED: "outline",
  FAILED: "destructive",
  FAILING: "destructive",
  UNKNOWN: "ghost",
  INVALIDATING: "secondary",
  SUCCEEDING: "secondary",
}

function stateVariant(state: string) {
  return STATE_VARIANT[state] ?? "ghost"
}

function formatTimestamp(value?: string | null) {
  if (!value) return "—"
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

function formatRelative(value?: string | null) {
  if (!value) return "—"
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  const diffMs = Date.now() - date.getTime()
  const abs = Math.abs(diffMs)
  const minute = 60_000
  const hour = 60 * minute
  const day = 24 * hour
  if (abs < hour) return `${Math.round(diffMs / minute)}m ago`
  if (abs < day) return `${Math.round(diffMs / hour)}h ago`
  return `${Math.round(diffMs / day)}d ago`
}

export function PipelinesView({ initialPipelines, initialError }: Props) {
  const [pipelines, setPipelines] = useState<Pipeline[]>(initialPipelines)
  const [error, setError] = useState<string | undefined>(initialError)
  const [filter, setFilter] = useState("")
  const [selected, setSelected] = useState<PipelineDetail | null>(null)
  const [sheetOpen, setSheetOpen] = useState(false)
  const [isRefreshing, startRefresh] = useTransition()
  const [isLoadingDetail, startLoadDetail] = useTransition()

  const filtered = useMemo(() => {
    const term = filter.trim().toLowerCase()
    if (!term) return pipelines
    return pipelines.filter((p) =>
      [p.name, p.namespace, p.state, p.spark_version ?? ""]
        .join(" ")
        .toLowerCase()
        .includes(term),
    )
  }, [filter, pipelines])

  const counts = useMemo(() => {
    const result: Record<string, number> = {}
    for (const pipeline of pipelines) {
      result[pipeline.state] = (result[pipeline.state] ?? 0) + 1
    }
    return result
  }, [pipelines])

  const refresh = () => {
    startRefresh(async () => {
      try {
        const { items } = await listPipelines()
        setPipelines(items)
        setError(undefined)
      } catch (err) {
        setError(err instanceof Error ? err.message : "Unknown error")
      }
    })
  }

  const openDetail = (pipeline: Pipeline) => {
    setSelected({ ...pipeline, raw: null })
    setSheetOpen(true)
    startLoadDetail(async () => {
      try {
        const detail = await getPipeline(pipeline.namespace, pipeline.name)
        setSelected(detail)
      } catch (err) {
        setError(err instanceof Error ? err.message : "Unknown error")
      }
    })
  }

  return (
    <div className="flex flex-col gap-4 p-6">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold tracking-tight">Pipelines</h1>
          <p className="text-xs text-muted-foreground">
            SparkApplications across the cluster, backed by the control plane.
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={refresh}
          disabled={isRefreshing}
        >
          <IconRefresh className="size-3.5" />
          Refresh
        </Button>
      </header>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-[220px] max-w-sm">
          <IconSearch className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
            placeholder="Filter by name, namespace, state…"
            className="h-8 pl-7 text-xs"
          />
        </div>
        <div className="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
          <span>
            {filtered.length}/{pipelines.length}
          </span>
          {Object.entries(counts).map(([state, count]) => (
            <Badge key={state} variant={stateVariant(state)}>
              {state} · {count}
            </Badge>
          ))}
        </div>
      </div>

      {error ? (
        <div className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          {error}
        </div>
      ) : null}

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[220px]">Name</TableHead>
              <TableHead>Namespace</TableHead>
              <TableHead>State</TableHead>
              <TableHead>Executors</TableHead>
              <TableHead>Spark</TableHead>
              <TableHead>Submitted</TableHead>
              <TableHead>Age</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {filtered.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-16 text-center text-muted-foreground"
                >
                  No pipelines found.
                </TableCell>
              </TableRow>
            ) : (
              filtered.map((pipeline) => {
                const exec = pipeline.executor_state_summary
                return (
                  <TableRow
                    key={`${pipeline.namespace}/${pipeline.name}`}
                    className="cursor-pointer"
                    onClick={() => openDetail(pipeline)}
                  >
                    <TableCell className="font-mono text-xs">
                      {pipeline.name}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {pipeline.namespace}
                    </TableCell>
                    <TableCell>
                      <Badge variant={stateVariant(pipeline.state)}>
                        {pipeline.state}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs">
                      <span className="text-emerald-500">{exec.running}R</span>
                      <span className="text-muted-foreground"> · </span>
                      <span>{exec.completed}C</span>
                      <span className="text-muted-foreground"> · </span>
                      <span className="text-destructive">{exec.failed}F</span>
                      {exec.pending > 0 ? (
                        <>
                          <span className="text-muted-foreground"> · </span>
                          <span>{exec.pending}P</span>
                        </>
                      ) : null}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {pipeline.spark_version ?? "—"}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {formatTimestamp(pipeline.submission_time)}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {formatRelative(pipeline.creation_timestamp)}
                    </TableCell>
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
      </div>

      <Sheet open={sheetOpen} onOpenChange={setSheetOpen}>
        <SheetContent className="flex w-full flex-col gap-4 sm:max-w-xl">
          <SheetHeader>
            <SheetTitle className="font-mono text-sm">
              {selected?.name}
            </SheetTitle>
            <SheetDescription className="text-xs">
              {selected?.namespace} · {selected?.state}
            </SheetDescription>
          </SheetHeader>
          {selected ? (
            <div className="flex flex-col gap-3 overflow-auto text-xs">
              <DetailGrid pipeline={selected} />
              {selected.application_state_message ? (
                <div className="rounded-md border border-destructive/40 bg-destructive/5 p-2 text-destructive">
                  {selected.application_state_message}
                </div>
              ) : null}
              <section className="flex flex-col gap-1">
                <div className="text-[0.65rem] font-medium uppercase tracking-wide text-muted-foreground">
                  Raw manifest
                </div>
                <pre className="max-h-[50vh] overflow-auto rounded-md border bg-muted/40 p-2 font-mono text-[0.65rem] leading-relaxed">
                  {isLoadingDetail && !selected.raw
                    ? "Loading…"
                    : JSON.stringify(selected.raw ?? {}, null, 2)}
                </pre>
              </section>
            </div>
          ) : null}
        </SheetContent>
      </Sheet>
    </div>
  )
}

function DetailGrid({ pipeline }: { pipeline: Pipeline }) {
  const rows: [string, string][] = [
    ["Type", pipeline.application_type ?? "—"],
    ["Mode", pipeline.mode ?? "—"],
    ["Spark", pipeline.spark_version ?? "—"],
    ["Image", pipeline.image ?? "—"],
    ["Main file", pipeline.main_application_file ?? "—"],
    ["Driver pod", pipeline.driver_pod_name ?? "—"],
    [
      "Executors",
      pipeline.executor_instances != null
        ? String(pipeline.executor_instances)
        : "—",
    ],
    ["Submission attempts", String(pipeline.submission_attempts ?? "—")],
    ["Execution attempts", String(pipeline.execution_attempts ?? "—")],
    ["Submission time", formatTimestamp(pipeline.submission_time)],
    ["Termination time", formatTimestamp(pipeline.termination_time)],
    ["Created", formatTimestamp(pipeline.creation_timestamp)],
  ]
  return (
    <dl className="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-1">
      {rows.map(([label, value]) => (
        <div key={label} className="contents">
          <dt className="text-muted-foreground">{label}</dt>
          <dd className="truncate font-mono">{value}</dd>
        </div>
      ))}
    </dl>
  )
}
