"use client"

import { useMemo } from "react"
import type { ComputeMetricsResponse } from "@/app/(app)/pipelines/actions"
import {
  type ChartConfig,
  EChartsLineChart,
} from "@/components/evilcharts/charts/echarts-line-chart"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

type MetricDefinition = ComputeMetricsResponse["definitions"][number]
type ChartRow = Record<string, string | number | null>

type MetricGroup = {
  id: string
  title: string
  description: string
  unit: string
  definitions: MetricDefinition[]
}

const palette = [
  { light: "#2563eb", dark: "#60a5fa" },
  { light: "#0891b2", dark: "#22d3ee" },
  { light: "#7c3aed", dark: "#a78bfa" },
  { light: "#d97706", dark: "#fbbf24" },
]

const metricLabels: Record<string, string> = {
  "mizuumi.compute.operation.duration": "Operation duration",
  "mizuumi.compute.operation.cpu.time": "CPU time",
  "mizuumi.compute.operation.input.rows": "Input rows",
  "mizuumi.compute.operation.output.rows": "Output rows",
  "mizuumi.compute.operation.input.bytes": "Input bytes",
  "mizuumi.compute.operation.output.bytes": "Output bytes",
  "mizuumi.compute.operation.memory.peak": "Peak memory",
  "mizuumi.compute.operation.spill": "Spill",
  "mizuumi.compute.operation.count": "Operations",
  "duckdb.query.duration": "Query duration",
  "duckdb.query.cpu.time": "CPU time",
  "duckdb.query.blocked_thread.time": "Blocked thread time",
  "duckdb.query.rows.returned": "Rows returned",
  "duckdb.query.rows.scanned": "Rows scanned",
  "duckdb.query.cardinality.cumulative": "Cumulative cardinality",
  "duckdb.query.result.size": "Result size",
  "duckdb.query.memory.peak_buffer": "Peak buffer memory",
  "duckdb.query.temp_directory.peak_size": "Peak temp storage",
  "duckdb.query.memory.allocated": "Allocated memory",
  "duckdb.query.io.read": "Bytes read",
  "duckdb.query.io.written": "Bytes written",
  "flare.task.duration": "Task duration",
  "flare.task.records_throughput": "Record throughput",
  "flare.task.shuffle.read_bytes": "Task shuffle read",
  "flare.task.shuffle.write_bytes": "Task shuffle write",
  "flare.stage.executor.run_time": "Stage executor time",
  "flare.stage.input.bytes": "Stage input",
  "flare.stage.output.bytes": "Stage output",
  "flare.stage.shuffle.read_bytes": "Stage shuffle read",
  "flare.stage.shuffle.write_bytes": "Stage shuffle write",
}

function metricLabel(name: string) {
  return (
    metricLabels[name] ??
    name
      .replace(/^duckdb\.query\./, "")
      .replace(/^flare\./, "")
      .replace(/^mizuumi\.compute\.operation\./, "")
      .replaceAll(".", " ")
      .replaceAll("_", " ")
  )
}

function formatNumber(value: number, unit: string) {
  if (unit === "By") {
    return new Intl.NumberFormat(undefined, {
      notation: "compact",
      style: "unit",
      unit: "byte",
      unitDisplay: "narrow",
      maximumFractionDigits: 1,
    }).format(value)
  }

  if (unit === "ms") {
    return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 }).format(value)} ms`
  }

  if (unit === "{records}/s") {
    return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 }).format(value)}/s`
  }

  return new Intl.NumberFormat(undefined, {
    notation: Math.abs(value) >= 10_000 ? "compact" : "standard",
    maximumFractionDigits: 2,
  }).format(value)
}

function axisFormatter(unit: string) {
  return (value: number) => formatNumber(value, unit)
}

function groupDefinitions(definitions: MetricDefinition[]): MetricGroup[] {
  const comparable = definitions.filter((definition) => definition.comparable)
  const diagnostics = definitions.filter((definition) => !definition.comparable)
  const groups: (Omit<MetricGroup, "definitions"> & {
    source: MetricDefinition[]
  })[] = [
    {
      id: "comparison-time",
      title: "Comparable · execution time",
      description: "Canonical wall-clock and CPU measurements",
      unit: "ms",
      source: comparable,
    },
    {
      id: "comparison-rows",
      title: "Comparable · rows",
      description: "Canonical input and output row counts",
      unit: "{row}",
      source: comparable,
    },
    {
      id: "comparison-bytes",
      title: "Comparable · bytes & memory",
      description: "Canonical I/O, peak memory, and spill volume",
      unit: "By",
      source: comparable,
    },
    {
      id: "comparison-reliability",
      title: "Comparable · reliability",
      description: "Completed operations in each interval",
      unit: "{operation}",
      source: comparable,
    },
    {
      id: "diagnostic-time",
      title: "Engine diagnostics · execution time",
      description: "Engine-specific query, stage, and task timing",
      unit: "ms",
      source: diagnostics,
    },
    {
      id: "diagnostic-throughput",
      title: "Engine diagnostics · throughput",
      description: "Engine-specific rows and records per second",
      unit: "count",
      source: diagnostics,
    },
    {
      id: "diagnostic-memory",
      title: "Engine diagnostics · memory & storage",
      description: "Engine-specific result, allocation, and temporary storage",
      unit: "By",
      source: diagnostics,
    },
    {
      id: "diagnostic-io",
      title: "Engine diagnostics · I/O",
      description: "Engine-specific storage and shuffle volume",
      unit: "By",
      source: diagnostics,
    },
  ]

  return groups
    .map((group) => {
      const matchingDefinitions = group.source.filter((definition) => {
        if (group.id === "comparison-time") {
          return definition.unit === "s" || definition.unit === "ms"
        }
        if (group.id === "comparison-rows") {
          return definition.unit === "{row}"
        }
        if (group.id === "comparison-bytes") {
          return definition.unit === "By"
        }
        if (group.id === "comparison-reliability") {
          return definition.unit === "{operation}"
        }
        if (group.id === "diagnostic-time") {
          return definition.unit === "s" || definition.unit === "ms"
        }
        if (group.id === "diagnostic-throughput") {
          return (
            definition.unit === "{row}" || definition.unit === "{records}/s"
          )
        }
        if (group.id === "diagnostic-io") {
          return (
            definition.name.includes(".io.") ||
            (definition.name.startsWith("flare.") && definition.unit === "By")
          )
        }
        if (group.id === "diagnostic-memory") {
          return (
            definition.unit === "By" &&
            !definition.name.includes(".io.") &&
            !definition.name.startsWith("flare.")
          )
        }
        return false
      })
      return {
        ...group,
        unit:
          group.id === "diagnostic-throughput"
            ? (matchingDefinitions[0]?.unit ?? group.unit)
            : group.unit,
        definitions: matchingDefinitions,
      }
    })
    .filter((group) => group.definitions.length > 0)
}

function buildChartRows(metrics: ComputeMetricsResponse) {
  const rows = new Map<number, ChartRow>()
  const results = metrics.data?.data?.data?.results ?? []
  const definitions = new Map(
    metrics.definitions.map((definition) => [
      definition.query_name,
      definition,
    ]),
  )

  for (const result of results) {
    for (const aggregation of result.aggregations ?? []) {
      for (const series of aggregation.series ?? []) {
        for (const point of series.values ?? []) {
          const row = rows.get(point.timestamp) ?? {
            timestamp: point.timestamp,
            time: new Date(point.timestamp).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            }),
          }
          row[result.queryName] =
            definitions.get(result.queryName)?.unit === "s"
              ? point.value * 1_000
              : point.value
          rows.set(point.timestamp, row)
        }
      }
    }
  }

  return [...rows.values()].sort(
    (left, right) => Number(left.timestamp) - Number(right.timestamp),
  )
}

function latestValue(rows: ChartRow[], queryName: string) {
  for (let index = rows.length - 1; index >= 0; index -= 1) {
    const value = rows[index]?.[queryName]
    if (typeof value === "number") return value
  }
  return null
}

function chartConfig(definitions: MetricDefinition[]): ChartConfig {
  return Object.fromEntries(
    definitions.map((definition, index) => {
      const color = palette[index % palette.length]
      return [
        definition.query_name,
        {
          label: metricLabel(definition.name),
          colors: {
            light: [color.light],
            dark: [color.dark],
          },
        },
      ]
    }),
  )
}

export function MetricsView({ metrics }: { metrics: ComputeMetricsResponse }) {
  const rows = useMemo(() => buildChartRows(metrics), [metrics])
  const groups = useMemo(
    () => groupDefinitions(metrics.definitions),
    [metrics.definitions],
  )

  if (rows.length === 0) {
    return (
      <div className="rounded-md border px-4 py-10 text-center text-sm text-muted-foreground">
        No metric samples were returned for this time range.
      </div>
    )
  }

  return (
    <div className="grid gap-4 xl:grid-cols-2">
      {groups.map((group) => {
        const config = chartConfig(group.definitions)

        return (
          <Card key={group.id}>
            <CardHeader>
              <CardTitle>{group.title}</CardTitle>
              <CardDescription>{group.description}</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4">
              <dl className="grid grid-cols-2 gap-2 sm:grid-cols-3">
                {group.definitions.map((definition) => {
                  const value = latestValue(rows, definition.query_name)
                  return (
                    <div
                      key={definition.query_name}
                      className="min-w-0 rounded-md bg-muted/50 px-3 py-2"
                    >
                      <dt className="truncate text-[11px] text-muted-foreground">
                        {metricLabel(definition.name)}
                      </dt>
                      <dd className="mt-0.5 truncate font-mono text-sm font-medium tabular-nums">
                        {value === null ? "—" : formatNumber(value, group.unit)}
                      </dd>
                    </div>
                  )
                })}
              </dl>

              <EChartsLineChart
                className="h-72 w-full"
                data={rows}
                config={config}
                xDataKey="time"
                curveType="monotone"
                enableHoverHighlight
              >
                <EChartsLineChart.Grid />
                <EChartsLineChart.XAxis dataKey="time" hideDots />
                <EChartsLineChart.YAxis
                  hideDots
                  tickFormatter={axisFormatter(group.unit)}
                />
                <EChartsLineChart.Legend
                  align="left"
                  verticalAlign="top"
                  isClickable
                />
                <EChartsLineChart.Tooltip cursor roundness="md" />
                {rows.length > 12 ? (
                  <EChartsLineChart.Brush height={44} />
                ) : null}
                {group.definitions.map((definition) => (
                  <EChartsLineChart.Line
                    key={definition.query_name}
                    dataKey={definition.query_name}
                    strokeWidth={1.5}
                    isClickable
                  >
                    {rows.length <= 12 ? (
                      <EChartsLineChart.Dot variant="border" />
                    ) : null}
                    <EChartsLineChart.ActiveDot variant="colored-border" />
                  </EChartsLineChart.Line>
                ))}
              </EChartsLineChart>
            </CardContent>
          </Card>
        )
      })}
    </div>
  )
}
