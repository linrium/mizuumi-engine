import { IconCpu } from "@tabler/icons-react"
import Link from "next/link"
import type { Compute } from "@/app/(app)/pipelines/actions"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"

type Props = {
  computes: Compute[]
  error?: string
}

function formatTimestamp(value?: string | null) {
  if (!value) return "—"
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function ComputeView({ computes, error }: Props) {
  const duckComputes = computes.filter(
    (compute) => compute.kind === "duck_server",
  )
  const sparkComputes = computes.filter(
    (compute) => compute.kind === "spark_application",
  )

  return (
    <div className="flex flex-col gap-4">
      {error ? (
        <div className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          {error}
        </div>
      ) : null}

      {computes.length === 0 ? (
        <div className="rounded-md border px-4 py-8 text-center text-sm text-muted-foreground">
          No compute instances found.
        </div>
      ) : (
        <div className="flex flex-col gap-6">
          {sparkComputes.length > 0 ? (
            <ComputeGroup title="Spark applications" computes={sparkComputes} />
          ) : null}
          {duckComputes.length > 0 ? (
            <ComputeGroup title="Duck Server" computes={duckComputes} />
          ) : null}
        </div>
      )}
    </div>
  )
}

function ComputeGroup({
  title,
  computes,
}: {
  title: string
  computes: Compute[]
}) {
  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-semibold">{title}</h2>
        <Badge variant="outline">{computes.length}</Badge>
      </div>
      <div className="grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
        {computes.map((compute) => (
          <Link
            key={compute.id}
            href={`/pipelines/computes/${encodeURIComponent(compute.id)}`}
          >
            <Card className="h-full transition-colors hover:bg-muted/40">
              <CardHeader>
                <div className="flex items-start justify-between gap-3">
                  <div className="flex min-w-0 items-center gap-2">
                    <IconCpu className="size-4 shrink-0 text-muted-foreground" />
                    <CardTitle className="truncate font-mono text-sm">
                      {compute.name}
                    </CardTitle>
                  </div>
                  <Badge variant={compute.ready ? "default" : "secondary"}>
                    {compute.ready ? "Ready" : compute.phase}
                  </Badge>
                </div>
                <CardDescription className="font-mono text-xs">
                  {compute.namespace}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <dl className="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-1 text-xs">
                  {compute.kind === "spark_application" ? (
                    <>
                      <dt className="text-muted-foreground">Spark</dt>
                      <dd>{compute.spark_version ?? "—"}</dd>
                      <dt className="text-muted-foreground">Driver</dt>
                      <dd className="truncate font-mono">
                        {compute.driver_pod_name ?? "—"}
                      </dd>
                      <dt className="text-muted-foreground">Executors</dt>
                      <dd>{compute.executor_instances ?? "—"}</dd>
                    </>
                  ) : (
                    <>
                      <dt className="text-muted-foreground">Pod IP</dt>
                      <dd className="truncate font-mono">
                        {compute.pod_ip ?? "—"}
                      </dd>
                      <dt className="text-muted-foreground">Node</dt>
                      <dd className="truncate font-mono">
                        {compute.node_name ?? "—"}
                      </dd>
                      <dt className="text-muted-foreground">Restarts</dt>
                      <dd>{compute.restart_count}</dd>
                    </>
                  )}
                  <dt className="text-muted-foreground">Started</dt>
                  <dd>{formatTimestamp(compute.started_at)}</dd>
                </dl>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </section>
  )
}
