"use client"

import {
  IconAlertTriangle,
  IconBraces,
  IconCircleCheck,
  IconLoader2,
  IconRefresh,
} from "@tabler/icons-react"
import { useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { reconcileDataContract } from "./actions"
import type {
  ContractTarget,
  DataContract,
  DataContractDifference,
  OpenDataContractDocument,
} from "./types"

function formatDate(value?: string) {
  if (!value) return "—"
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

function description(document: OpenDataContractDocument) {
  if (typeof document.description === "string") return document.description
  const purpose = document.description?.purpose
  return typeof purpose === "string" ? purpose : undefined
}

function statusVariant(state: string): "secondary" | "destructive" | "outline" {
  if (state === "IN_SYNC") return "secondary"
  if (state === "DRIFTED" || state === "TABLE_MISSING") return "destructive"
  return "outline"
}

function jsonValue(value: unknown) {
  if (value === null || value === undefined) return "—"
  return typeof value === "string" ? value : JSON.stringify(value, null, 2)
}

function Difference({ difference }: { difference: DataContractDifference }) {
  return (
    <div className="space-y-2 rounded-md border border-destructive/30 bg-destructive/5 p-3">
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant="destructive">{difference.severity ?? "ERROR"}</Badge>
        <span className="font-mono text-xs font-medium">
          {difference.code ?? "SCHEMA_DIFFERENCE"}
        </span>
        {difference.path ? (
          <span className="font-mono text-xs text-muted-foreground">
            {difference.path}
          </span>
        ) : null}
      </div>
      <div className="grid gap-2 md:grid-cols-2">
        <div>
          <div className="mb-1 text-[0.625rem] font-medium uppercase tracking-wide text-muted-foreground">
            Contract
          </div>
          <pre className="overflow-x-auto whitespace-pre-wrap rounded bg-background p-2 font-mono text-[0.6875rem]">
            {jsonValue(difference.expected)}
          </pre>
        </div>
        <div>
          <div className="mb-1 text-[0.625rem] font-medium uppercase tracking-wide text-muted-foreground">
            Unity Catalog
          </div>
          <pre className="overflow-x-auto whitespace-pre-wrap rounded bg-background p-2 font-mono text-[0.6875rem]">
            {jsonValue(difference.actual)}
          </pre>
        </div>
      </div>
    </div>
  )
}

export function DataContractView({
  target,
  initialContract,
  initialError,
}: {
  target: ContractTarget
  initialContract: DataContract | null
  initialError?: string
}) {
  const [contract, setContract] = useState(initialContract)
  const [message, setMessage] = useState<
    { kind: "error" | "success"; text: string } | undefined
  >(initialError ? { kind: "error", text: initialError } : undefined)
  const [isPending, startTransition] = useTransition()

  const reconcile = () => {
    setMessage(undefined)
    startTransition(async () => {
      const result = await reconcileDataContract(target)
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setContract(result.data)
      setMessage({
        kind: "success",
        text:
          result.data?.sync_state === "IN_SYNC"
            ? "The contract matches the Unity Catalog table."
            : `Reconciliation completed with state ${result.data?.sync_state ?? "UNKNOWN"}.`,
      })
    })
  }

  if (!contract) {
    return (
      <div className="space-y-3">
        {message?.kind === "error" ? (
          <div className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
            <IconAlertTriangle className="mt-0.5 size-4 shrink-0" />
            {message.text}
          </div>
        ) : null}
        <Card>
          <CardHeader>
            <CardTitle>
              {message?.kind === "error"
                ? "Data contract unavailable"
                : "No active data contract"}
            </CardTitle>
            <CardDescription>
              {message?.kind === "error"
                ? "Studio could not read this table's contract from the Control Plane. Reload the page after the service is available."
                : "This table does not have an Open Data Contract Standard v3.1.0 document in the Control Plane yet. Spark jobs publish the contract before creating or updating the Delta table."}
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  const document = contract.document
  const contractSchema = document.schema?.[0]
  const properties = contractSchema?.properties ?? []
  const differences = Array.isArray(contract.differences)
    ? contract.differences
    : []

  return (
    <div className="space-y-4">
      {message ? (
        <div
          className={
            message.kind === "error"
              ? "flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive"
              : "flex items-start gap-2 rounded-md border border-emerald-500/40 bg-emerald-500/5 p-3 text-xs text-emerald-700 dark:text-emerald-400"
          }
        >
          {message.kind === "error" ? (
            <IconAlertTriangle className="mt-0.5 size-4 shrink-0" />
          ) : (
            <IconCircleCheck className="mt-0.5 size-4 shrink-0" />
          )}
          {message.text}
        </div>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle className="flex flex-wrap items-center gap-2">
            Open Data Contract
            <Badge variant="outline">{document.apiVersion ?? "v3.1.0"}</Badge>
            <Badge variant={statusVariant(contract.sync_state)}>
              {contract.sync_state.replaceAll("_", " ")}
            </Badge>
          </CardTitle>
          <CardDescription>
            {description(document) ??
              `Active contract for ${target.catalog}.${target.schema}.${target.table}.`}
          </CardDescription>
          <CardAction>
            <Button
              size="sm"
              variant="outline"
              disabled={isPending}
              onClick={reconcile}
            >
              {isPending ? (
                <IconLoader2 className="animate-spin" />
              ) : (
                <IconRefresh />
              )}
              Reconcile
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent>
          <dl className="grid gap-x-6 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
            {[
              ["Version", contract.version],
              ["Status", document.status ?? "—"],
              ["Domain", document.domain ?? "—"],
              ["Created by", contract.created_by],
              ["Published", formatDate(contract.created_at)],
              ["Last checked", formatDate(contract.checked_at)],
              ["Contract ID", contract.id],
              ["Unity table ID", contract.unity_table_id ?? "—"],
            ].map(([label, value]) => (
              <div key={label} className="min-w-0">
                <dt className="text-[0.625rem] font-medium uppercase tracking-wide text-muted-foreground">
                  {label}
                </dt>
                <dd className="mt-0.5 truncate font-mono text-xs" title={value}>
                  {value}
                </dd>
              </div>
            ))}
          </dl>
        </CardContent>
      </Card>

      <section>
        <div className="mb-2 flex items-end justify-between gap-3">
          <div>
            <h3 className="text-xs font-semibold">Contract schema</h3>
            <p className="text-xs text-muted-foreground">
              {contractSchema?.physicalName ?? document.name}
            </p>
          </div>
          <span className="text-xs text-muted-foreground">
            {properties.length} {properties.length === 1 ? "column" : "columns"}
          </span>
        </div>
        <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
          <table className="w-full text-xs">
            <thead className="bg-muted/50 text-muted-foreground">
              <tr>
                <th className="px-3 py-2 text-left font-medium">Name</th>
                <th className="px-3 py-2 text-left font-medium">
                  Logical type
                </th>
                <th className="px-3 py-2 text-left font-medium">
                  Physical type
                </th>
                <th className="px-3 py-2 text-left font-medium">Required</th>
                <th className="px-3 py-2 text-left font-medium">Description</th>
              </tr>
            </thead>
            <tbody>
              {properties.map((property, index) => (
                <tr
                  key={property.id ?? `${property.name}-${index}`}
                  className="border-t"
                >
                  <td className="px-3 py-1.5 font-mono">
                    {property.name ?? "—"}
                  </td>
                  <td className="px-3 py-1.5">{property.logicalType ?? "—"}</td>
                  <td className="px-3 py-1.5">
                    <code className="rounded bg-muted px-1.5 py-0.5 text-[0.7rem]">
                      {property.physicalType ?? "—"}
                    </code>
                  </td>
                  <td className="px-3 py-1.5 text-muted-foreground">
                    {property.required ? "yes" : "no"}
                  </td>
                  <td className="px-3 py-1.5 text-muted-foreground">
                    {property.description ?? "—"}
                  </td>
                </tr>
              ))}
              {properties.length === 0 ? (
                <tr>
                  <td
                    colSpan={5}
                    className="px-3 py-6 text-center text-muted-foreground"
                  >
                    No schema properties in this contract.
                  </td>
                </tr>
              ) : null}
            </tbody>
          </table>
        </div>
      </section>

      <Card>
        <CardHeader>
          <CardTitle>Unity Catalog reconciliation</CardTitle>
          <CardDescription>
            Comparison of the active contract schema and metadata properties
            against the current Unity Catalog table.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {differences.length === 0 ? (
            <div className="flex items-center gap-2 rounded-md border border-emerald-500/30 bg-emerald-500/5 p-3 text-emerald-700 dark:text-emerald-400">
              <IconCircleCheck className="size-4" />
              No differences were reported.
            </div>
          ) : (
            differences.map((difference, index) => (
              <Difference
                key={`${difference.code}-${difference.path}-${index}`}
                difference={difference}
              />
            ))
          )}
          <dl className="grid gap-3 border-t pt-3 sm:grid-cols-2">
            <div>
              <dt className="text-[0.625rem] font-medium uppercase tracking-wide text-muted-foreground">
                Contract schema hash
              </dt>
              <dd className="mt-1 break-all font-mono text-[0.6875rem]">
                {contract.schema_hash}
              </dd>
            </div>
            <div>
              <dt className="text-[0.625rem] font-medium uppercase tracking-wide text-muted-foreground">
                Observed schema hash
              </dt>
              <dd className="mt-1 break-all font-mono text-[0.6875rem]">
                {contract.observed_schema_hash ?? "—"}
              </dd>
            </div>
          </dl>
        </CardContent>
      </Card>

      <details className="group rounded-lg ring-1 ring-foreground/10">
        <summary className="flex cursor-pointer list-none items-center gap-2 px-4 py-3 text-xs font-medium">
          <IconBraces className="size-4" />
          Raw ODCS document
        </summary>
        <pre className="max-h-[36rem] overflow-auto border-t bg-muted/20 p-4 font-mono text-[0.6875rem] leading-relaxed">
          {JSON.stringify(document, null, 2)}
        </pre>
      </details>
    </div>
  )
}
