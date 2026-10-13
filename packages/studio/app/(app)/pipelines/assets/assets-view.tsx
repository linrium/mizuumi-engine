import { Badge } from "@/components/ui/badge"
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
              <TableHead className="w-[320px]">Asset</TableHead>
              <TableHead>Group</TableHead>
              <TableHead>Kinds</TableHead>
              <TableHead>Definition</TableHead>
              <TableHead>Repository</TableHead>
              <TableHead>Last event</TableHead>
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
                      <div>{asset.repository.name}</div>
                      <div className="text-muted-foreground">
                        {asset.repository.location.name}
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
