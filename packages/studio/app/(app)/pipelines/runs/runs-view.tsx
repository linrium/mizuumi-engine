import { Badge } from "@/components/ui/badge"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { DagsterRun } from "@/lib/dagster"

type Props = {
  initialRuns: DagsterRun[]
  initialError?: string
}

const STATUS_VARIANT: Record<
  string,
  "default" | "secondary" | "destructive" | "outline" | "ghost"
> = {
  QUEUED: "secondary",
  NOT_STARTED: "secondary",
  MANAGED: "secondary",
  STARTING: "secondary",
  STARTED: "default",
  SUCCESS: "outline",
  FAILURE: "destructive",
  CANCELING: "secondary",
  CANCELED: "ghost",
}

function formatTimestamp(value: number | null) {
  if (value == null) return "—"
  const date = new Date(value * 1000)
  return Number.isNaN(date.getTime()) ? String(value) : date.toLocaleString()
}

function formatDuration(run: DagsterRun) {
  if (run.startTime == null) return "—"
  const endTime = run.endTime ?? run.updateTime
  const seconds = Math.max(0, endTime - run.startTime)

  if (seconds < 60) return `${seconds.toFixed(1)}s`
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.round(seconds % 60)
  if (minutes < 60) return `${minutes}m ${remainingSeconds}s`
  const hours = Math.floor(minutes / 60)
  return `${hours}h ${minutes % 60}m`
}

function tagValue(run: DagsterRun, key: string) {
  return run.tags.find((tag) => tag.key === key)?.value
}

export function RunsView({ initialRuns, initialError }: Props) {
  return (
    <div className="flex flex-col gap-4">
      {initialError ? (
        <div className="rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          {initialError}
        </div>
      ) : null}

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Run ID</TableHead>
              <TableHead>Job</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Started</TableHead>
              <TableHead>Duration</TableHead>
              <TableHead>Repository</TableHead>
              <TableHead>Image</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialRuns.length === 0 ? (
              <TableRow>
                <TableCell
                  className="h-16 text-center text-muted-foreground"
                  colSpan={7}
                >
                  No runs found.
                </TableCell>
              </TableRow>
            ) : (
              initialRuns.map((run) => (
                <TableRow key={run.runId}>
                  <TableCell className="font-mono text-xs" title={run.runId}>
                    {run.runId.slice(0, 8)}
                  </TableCell>
                  <TableCell className="font-mono text-xs">
                    {run.pipelineName}
                  </TableCell>
                  <TableCell>
                    <Badge variant={STATUS_VARIANT[run.status] ?? "ghost"}>
                      {run.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {formatTimestamp(run.startTime)}
                  </TableCell>
                  <TableCell className="text-xs text-muted-foreground">
                    {formatDuration(run)}
                  </TableCell>
                  <TableCell className="text-xs">
                    {run.repositoryOrigin ? (
                      <>
                        <div>{run.repositoryOrigin.repositoryName}</div>
                        <div className="text-muted-foreground">
                          {run.repositoryOrigin.repositoryLocationName}
                        </div>
                      </>
                    ) : (
                      <span className="text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell className="max-w-[260px] truncate font-mono text-xs text-muted-foreground">
                    {tagValue(run, "dagster/image") ?? "—"}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
