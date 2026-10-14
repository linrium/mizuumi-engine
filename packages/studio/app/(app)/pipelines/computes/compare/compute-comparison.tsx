"use client"

import { useRouter } from "next/navigation"
import type {
  Compute,
  ComputeMetricsResponse,
} from "@/app/(app)/pipelines/actions"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

type Props = {
  computes: Compute[]
  left: Compute | null
  right: Compute | null
  leftMetrics: ComputeMetricsResponse | null
  rightMetrics: ComputeMetricsResponse | null
}

type ComparisonMetric = {
  name: string
  label: string
  unit: string
  preference: "lower" | "higher" | "context"
}

const comparisonMetrics: ComparisonMetric[] = [
  {
    name: "mizuumi.compute.operation.duration",
    label: "Operation duration",
    unit: "s",
    preference: "lower",
  },
  {
    name: "mizuumi.compute.operation.cpu.time",
    label: "CPU time",
    unit: "s",
    preference: "lower",
  },
  {
    name: "mizuumi.compute.operation.input.rows",
    label: "Input rows",
    unit: "{row}",
    preference: "context",
  },
  {
    name: "mizuumi.compute.operation.output.rows",
    label: "Output rows",
    unit: "{row}",
    preference: "context",
  },
  {
    name: "mizuumi.compute.operation.input.bytes",
    label: "Input bytes",
    unit: "By",
    preference: "context",
  },
  {
    name: "mizuumi.compute.operation.output.bytes",
    label: "Output bytes",
    unit: "By",
    preference: "context",
  },
  {
    name: "mizuumi.compute.operation.memory.peak",
    label: "Peak memory",
    unit: "By",
    preference: "lower",
  },
  {
    name: "mizuumi.compute.operation.spill",
    label: "Spill",
    unit: "By",
    preference: "lower",
  },
]

function latestMetric(metrics: ComputeMetricsResponse | null, name: string) {
  if (!metrics) return null
  const definition = metrics.definitions.find(
    (candidate) => candidate.name === name,
  )
  if (!definition) return null
  const result = metrics.data?.data?.data?.results?.find(
    (candidate) => candidate.queryName === definition.query_name,
  )
  const values = result?.aggregations
    ?.flatMap((aggregation) => aggregation.series ?? [])
    .flatMap((series) => series.values ?? [])
    .filter((point) => Number.isFinite(point.value))
  return values?.at(-1)?.value ?? null
}

function formatValue(value: number | null, unit: string) {
  if (value === null) return "Not reported"
  if (unit === "s") {
    return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 }).format(value * 1_000)} ms`
  }
  if (unit === "By") {
    return new Intl.NumberFormat(undefined, {
      notation: "compact",
      style: "unit",
      unit: "byte",
      unitDisplay: "narrow",
      maximumFractionDigits: 1,
    }).format(value)
  }
  return new Intl.NumberFormat(undefined, {
    notation: Math.abs(value) >= 10_000 ? "compact" : "standard",
    maximumFractionDigits: 1,
  }).format(value)
}

function estimatedThroughput(metrics: ComputeMetricsResponse | null) {
  const duration = latestMetric(metrics, "mizuumi.compute.operation.duration")
  const rows = latestMetric(metrics, "mizuumi.compute.operation.output.rows")
  return duration && rows !== null ? rows / duration : null
}

function selectorLabel(compute: Compute) {
  return `${compute.kind === "spark_application" ? "Spark" : "DuckDB"} · ${compute.name}`
}

export function ComputeComparison({
  computes,
  left,
  right,
  leftMetrics,
  rightMetrics,
}: Props) {
  const router = useRouter()

  function navigate(leftId: string | null, rightId: string | null) {
    const params = new URLSearchParams()
    if (leftId) params.set("left", leftId)
    if (rightId) params.set("right", rightId)
    router.push(`/pipelines/computes/compare?${params.toString()}`)
  }

  if (!left || !right || !leftMetrics || !rightMetrics) {
    return (
      <div className="rounded-md border px-4 py-10 text-center text-sm text-muted-foreground">
        At least two computes are required for comparison.
      </div>
    )
  }

  const leftThroughput = estimatedThroughput(leftMetrics)
  const rightThroughput = estimatedThroughput(rightMetrics)
  const selections = [
    { compute: left, metrics: leftMetrics, throughput: leftThroughput },
    { compute: right, metrics: rightMetrics, throughput: rightThroughput },
  ]

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Comparison scope</CardTitle>
          <CardDescription>
            Select computes that ran the same logical workload, input snapshot,
            cache mode, and output semantics.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-2">
          <Select
            value={left.id}
            onValueChange={(value) => navigate(value, right.id)}
          >
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {computes.map((compute) => (
                <SelectItem key={compute.id} value={compute.id}>
                  {selectorLabel(compute)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={right.id}
            onValueChange={(value) => navigate(left.id, value)}
          >
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {computes.map((compute) => (
                <SelectItem key={compute.id} value={compute.id}>
                  {selectorLabel(compute)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <div className="grid gap-4 lg:grid-cols-2">
        {selections.map(
          ({
            compute: selectedCompute,
            metrics: selectedMetrics,
            throughput: selectedThroughput,
          }) => {
            return (
              <Card key={selectedCompute.id}>
                <CardHeader>
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <CardTitle className="font-mono text-sm">
                        {selectedCompute.name}
                      </CardTitle>
                      <CardDescription>
                        {selectedCompute.kind === "spark_application"
                          ? "Spark"
                          : "DuckDB"}
                      </CardDescription>
                    </div>
                    <Badge variant="outline">
                      {selectedMetrics.aggregation.toUpperCase()}
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent>
                  <dl className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-3 text-sm">
                    {comparisonMetrics.map((metric) => {
                      const value = latestMetric(selectedMetrics, metric.name)
                      return (
                        <div key={metric.name} className="contents">
                          <dt className="text-muted-foreground">
                            {metric.label}
                            {metric.preference !== "context" ? (
                              <span className="ml-1 text-[10px] uppercase">
                                {metric.preference} is better
                              </span>
                            ) : null}
                          </dt>
                          <dd className="font-mono font-medium tabular-nums">
                            {formatValue(value, metric.unit)}
                          </dd>
                        </div>
                      )
                    })}
                    <div className="contents">
                      <dt className="text-muted-foreground">
                        Estimated output throughput
                      </dt>
                      <dd className="font-mono font-medium tabular-nums">
                        {selectedThroughput === null
                          ? "Not reported"
                          : `${formatValue(selectedThroughput, "{row}")} rows/s`}
                      </dd>
                    </div>
                  </dl>
                </CardContent>
              </Card>
            )
          },
        )}
      </div>

      <p className="text-xs text-muted-foreground">
        “Not reported” means the engine cannot currently provide that canonical
        measurement. Estimated throughput divides the latest output-row maximum
        by the latest duration maximum and is directional, not an exact per-run
        ratio. Engine-native task, stage, and query diagnostics remain on each
        compute detail page.
      </p>
    </div>
  )
}
