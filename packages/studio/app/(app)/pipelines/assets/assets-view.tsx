"use client"

import { IconLoader2, IconPlayerPlay } from "@tabler/icons-react"
import { useState, useTransition } from "react"
import { materializeAsset } from "@/app/(app)/pipelines/assets/actions"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import type { DagsterAsset } from "@/lib/dagster"

type Props = {
  initialAssets: DagsterAsset[]
  initialError?: string
}

function assetName(asset: DagsterAsset) {
  return asset.assetKey.path.join(" / ")
}

function latestEvent(asset: DagsterAsset) {
  const events = [
    ...asset.assetMaterializations.map((event) => ({
      ...event,
      label: "Materialized",
    })),
    ...asset.assetObservations.map((event) => ({
      ...event,
      label: "Observed",
    })),
  ]

  return events.sort(
    (left, right) => Number(right.timestamp) - Number(left.timestamp),
  )[0]
}

function formatTimestamp(value?: string) {
  if (!value) return "—"
  const numericValue = Number(value)
  const date = new Date(Number.isNaN(numericValue) ? value : numericValue)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function AssetsView({ initialAssets, initialError }: Props) {
  const [feedback, setFeedback] = useState<
    { kind: "error" | "success"; message: string } | undefined
  >(initialError ? { kind: "error", message: initialError } : undefined)
  const [pendingAsset, setPendingAsset] = useState<string>()
  const [isPending, startTransition] = useTransition()

  const materialize = (asset: DagsterAsset) => {
    const key = asset.assetKey.path.join("\u0000")
    setPendingAsset(key)
    setFeedback(undefined)
    startTransition(async () => {
      try {
        const { runId } = await materializeAsset(
          asset.assetKey.path,
          asset.repository.name,
          asset.repository.location.name,
        )
        setFeedback({
          kind: "success",
          message: `Materialization launched for ${assetName(asset)} (run ${runId}).`,
        })
      } catch (error) {
        setFeedback({
          kind: "error",
          message: error instanceof Error ? error.message : "Unknown error",
        })
      } finally {
        setPendingAsset(undefined)
      }
    })
  }

  return (
    <div className="flex flex-col gap-4">
      {feedback ? (
        <div
          className={
            feedback.kind === "error"
              ? "rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive"
              : "rounded-md border border-emerald-500/40 bg-emerald-500/5 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-400"
          }
        >
          {feedback.message}
        </div>
      ) : null}

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[320px]">Asset</TableHead>
              <TableHead>Group</TableHead>
              <TableHead>Kinds</TableHead>
              <TableHead>Definition</TableHead>
              <TableHead>Last event</TableHead>
              <TableHead className="w-[120px] text-right">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialAssets.length === 0 ? (
              <TableRow>
                <TableCell
                  className="h-16 text-center text-muted-foreground"
                  colSpan={6}
                >
                  No assets found.
                </TableCell>
              </TableRow>
            ) : (
              initialAssets.map((asset) => {
                const event = latestEvent(asset)
                return (
                  <TableRow key={asset.assetKey.path.join("\u0000")}>
                    <TableCell>
                      <div className="font-mono text-xs">
                        {assetName(asset)}
                      </div>
                      {asset.description ? (
                        <div className="mt-0.5 max-w-[30rem] truncate text-xs text-muted-foreground">
                          {asset.description}
                        </div>
                      ) : null}
                    </TableCell>
                    <TableCell className="text-xs text-muted-foreground">
                      {asset.groupName || "—"}
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {asset.kinds.length > 0 ? (
                          asset.kinds.map((kind) => (
                            <Badge key={kind} variant="secondary">
                              {kind}
                            </Badge>
                          ))
                        ) : (
                          <span className="text-xs text-muted-foreground">
                            —
                          </span>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {asset.isMaterializable ? (
                          <Badge variant="outline">Materializable</Badge>
                        ) : null}
                        {asset.isObservable ? (
                          <Badge variant="outline">Observable</Badge>
                        ) : null}
                        {!asset.isMaterializable && !asset.isObservable ? (
                          <Badge variant="ghost">External</Badge>
                        ) : null}
                      </div>
                    </TableCell>
                    <TableCell className="text-xs">
                      {event ? (
                        <>
                          <div>{event.label}</div>
                          <div className="text-muted-foreground">
                            {formatTimestamp(event.timestamp)}
                          </div>
                        </>
                      ) : (
                        <span className="text-muted-foreground">—</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      {asset.isMaterializable ? (
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={isPending || asset.jobNames.length === 0}
                          onClick={() => materialize(asset)}
                          aria-label={`Materialize ${assetName(asset)}`}
                        >
                          {isPending &&
                          pendingAsset ===
                            asset.assetKey.path.join("\u0000") ? (
                            <IconLoader2 className="animate-spin" />
                          ) : (
                            <IconPlayerPlay />
                          )}
                          Materialize
                        </Button>
                      ) : (
                        <span className="text-xs text-muted-foreground">—</span>
                      )}
                    </TableCell>
                  </TableRow>
                )
              })
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
