"use client"

import {
  IconAlertTriangle,
  IconBraces,
  IconCircleCheck,
  IconClock,
  IconLoader2,
  IconPlayerPlay,
  IconRefresh,
  IconShieldCheck,
  IconUsers,
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
import { reconcileDataContract, runDataQuality } from "./actions"
import type {
  ContractTarget,
  DataContract,
  DataContractDifference,
  DataQualityRun,
  OpenDataContractDocument,
  OpenDataQualityRule,
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

const EXPECTATION_KEYS = [
  "mustBe",
  "mustNotBe",
  "mustBeGreaterThan",
  "mustBeGreaterOrEqualTo",
  "mustBeLessThan",
  "mustBeLessOrEqualTo",
  "mustBeBetween",
  "mustNotBeBetween",
]

function ruleExpectation(rule: OpenDataQualityRule) {
  const key = EXPECTATION_KEYS.find((candidate) => candidate in rule)
  return key ? `${key} ${jsonValue(rule[key])}` : "Declarative"
}

function qualityStatusVariant(status: string) {
  if (status === "PASSED") return "secondary" as const
  if (status === "FAILED" || status === "ERROR") return "destructive" as const
  return "outline" as const
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
  initialQualityRuns,
  initialQualityError,
}: {
  target: ContractTarget
  initialContract: DataContract | null
  initialError?: string
  initialQualityRuns: DataQualityRun[]
  initialQualityError?: string
}) {
  const [contract, setContract] = useState(initialContract)
  const [message, setMessage] = useState<
    { kind: "error" | "success"; text: string } | undefined
  >(initialError ? { kind: "error", text: initialError } : undefined)
  const [isPending, startTransition] = useTransition()
  const [qualityRuns, setQualityRuns] = useState(initialQualityRuns)
  const [qualityLoadError, setQualityLoadError] = useState(initialQualityError)
  const [isQualityPending, startQualityTransition] = useTransition()

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

  const runQuality = () => {
    setMessage(undefined)
    setQualityLoadError(undefined)
    startQualityTransition(async () => {
      const result = await runDataQuality(target)
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setQualityRuns((runs) => [
        result.data,
        ...runs.filter((run) => run.id !== result.data.id),
      ])
      setMessage({
        kind: result.data.status === "PASSED" ? "success" : "error",
        text: `Quality run ${result.data.status.toLowerCase()}: ${result.data.passed_count} passed, ${result.data.failed_count} failed, ${result.data.error_count} errors.`,
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
  const team = document.team
  const sla = document.slaProperties ?? []
  const customProperties = [
    ...(document.customProperties ?? []).map((property) => ({
      scope: "contract",
      property,
    })),
    ...(contractSchema?.customProperties ?? []).map((property) => ({
      scope: contractSchema?.name ?? "table",
      property,
    })),
    ...properties.flatMap((column) =>
      (column.customProperties ?? []).map((property) => ({
        scope: column.name ?? "column",
        property,
      })),
    ),
  ]
  const qualityRules = [
    ...((contractSchema?.quality ?? []).map((rule) => ({
      scope: contractSchema?.name ?? "table",
      rule,
    })) ?? []),
    ...properties.flatMap((property) =>
      (property.quality ?? []).map((rule) => ({
        scope: property.name ?? "column",
        rule,
      })),
    ),
  ]
  const latestQualityRun = qualityRuns[0]

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
              ["Data product", document.dataProduct ?? "—"],
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

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconUsers className="size-4" />
              Ownership
            </CardTitle>
            <CardDescription>
              {team?.description ?? "Accountable team and contract stewards."}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="font-medium">{team?.name ?? "—"}</div>
            {(team?.members ?? []).map((member) => (
              <div
                key={member.id ?? member.username}
                className="flex items-center justify-between gap-3 border-t pt-2"
              >
                <div className="min-w-0">
                  <div className="truncate font-mono text-xs">
                    {member.name ?? member.username}
                  </div>
                  {member.name ? (
                    <div className="truncate text-xs text-muted-foreground">
                      {member.username}
                    </div>
                  ) : null}
                </div>
                <Badge variant="outline">{member.role ?? "Member"}</Badge>
              </div>
            ))}
            {!team?.members?.length ? (
              <p className="text-muted-foreground">No team members declared.</p>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconShieldCheck className="size-4" />
              Governance
            </CardTitle>
            <CardDescription>
              Residency constraints and contract-specific properties.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {typeof document.description !== "string" &&
            typeof document.description?.limitations === "string" ? (
              <p>{document.description.limitations}</p>
            ) : null}
            <dl className="divide-y">
              {customProperties.map(({ scope, property }, index) => (
                <div
                  key={property.id ?? `${property.property}-${index}`}
                  className="grid grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] gap-3 py-2"
                >
                  <dt className="min-w-0 font-mono text-muted-foreground">
                    <span className="block truncate">
                      {property.property ?? "property"}
                    </span>
                    <span className="block text-[0.625rem]">{scope}</span>
                  </dt>
                  <dd className="break-words text-right font-mono">
                    {jsonValue(property.value)}
                  </dd>
                </div>
              ))}
            </dl>
            {customProperties.length === 0 ? (
              <p className="text-muted-foreground">
                No custom properties declared.
              </p>
            ) : null}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <IconClock className="size-4" />
            Service-level agreement
          </CardTitle>
          <CardDescription>
            Availability, freshness, retention, and response commitments from
            the contract.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {sla.map((item, index) => (
              <div
                key={item.id ?? `${item.property}-${index}`}
                className="rounded-md border p-3"
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium">{item.property}</span>
                  {item.driver ? (
                    <Badge variant="outline">{item.driver}</Badge>
                  ) : null}
                </div>
                <div className="mt-1 font-mono text-sm">
                  {jsonValue(item.value)} {item.unit ?? ""}
                </div>
                {item.element ? (
                  <div className="mt-1 font-mono text-xs text-muted-foreground">
                    {item.element}
                  </div>
                ) : null}
                {item.description ? (
                  <p className="mt-2 text-muted-foreground">
                    {item.description}
                  </p>
                ) : null}
              </div>
            ))}
          </div>
          {sla.length === 0 ? (
            <p className="text-muted-foreground">No SLA properties declared.</p>
          ) : null}
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
                    {property.primaryKey ? (
                      <Badge className="ml-2" variant="outline">
                        PK
                      </Badge>
                    ) : null}
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
          <CardTitle>Data quality</CardTitle>
          <CardDescription>
            ODCS rules executed as aggregate, read-only queries through DuckDB
            using your Unity Catalog access.
          </CardDescription>
          <CardAction>
            <Button
              size="sm"
              disabled={isQualityPending || qualityRules.length === 0}
              onClick={runQuality}
            >
              {isQualityPending ? (
                <IconLoader2 className="animate-spin" />
              ) : (
                <IconPlayerPlay />
              )}
              Run quality checks
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent className="space-y-4">
          {qualityLoadError ? (
            <div className="rounded-md border border-destructive/40 bg-destructive/5 p-3 text-destructive">
              {qualityLoadError}
            </div>
          ) : null}
          {latestQualityRun ? (
            <div className="flex flex-wrap items-center gap-2 rounded-md border p-3">
              <Badge variant={qualityStatusVariant(latestQualityRun.status)}>
                {latestQualityRun.status}
              </Badge>
              <span>{latestQualityRun.passed_count} passed</span>
              <span>{latestQualityRun.failed_count} failed</span>
              <span>{latestQualityRun.error_count} errors</span>
              <span className="ml-auto text-muted-foreground">
                {latestQualityRun.engine} ·{" "}
                {formatDate(
                  latestQualityRun.completed_at ?? latestQualityRun.started_at,
                )}
              </span>
            </div>
          ) : null}
          <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
            <table className="w-full text-xs">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-3 py-2 text-left font-medium">Rule</th>
                  <th className="px-3 py-2 text-left font-medium">Scope</th>
                  <th className="px-3 py-2 text-left font-medium">
                    Expectation
                  </th>
                  <th className="px-3 py-2 text-left font-medium">Observed</th>
                  <th className="px-3 py-2 text-left font-medium">Result</th>
                </tr>
              </thead>
              <tbody>
                {qualityRules.map(({ scope, rule }, index) => {
                  const result = latestQualityRun?.results.find(
                    (candidate) => candidate.rule_id === rule.id,
                  )
                  return (
                    <tr
                      key={rule.id ?? `${scope}-${index}`}
                      className="border-t"
                    >
                      <td className="px-3 py-2">
                        <div className="font-mono">
                          {rule.id ?? rule.name ?? "rule"}
                        </div>
                        {rule.description ? (
                          <div className="text-muted-foreground">
                            {rule.description}
                          </div>
                        ) : null}
                      </td>
                      <td className="px-3 py-2 font-mono">{scope}</td>
                      <td className="px-3 py-2 font-mono">
                        {ruleExpectation(rule)}
                      </td>
                      <td className="px-3 py-2 font-mono">
                        {result ? jsonValue(result.metric_value) : "—"}
                      </td>
                      <td className="px-3 py-2">
                        {result ? (
                          <Badge variant={qualityStatusVariant(result.status)}>
                            {result.status}
                          </Badge>
                        ) : (
                          <Badge variant="outline">NOT RUN</Badge>
                        )}
                      </td>
                    </tr>
                  )
                })}
                {qualityRules.length === 0 ? (
                  <tr>
                    <td
                      colSpan={5}
                      className="px-3 py-6 text-center text-muted-foreground"
                    >
                      No data-quality rules declared.
                    </td>
                  </tr>
                ) : null}
              </tbody>
            </table>
          </div>
          {qualityRuns.length > 1 ? (
            <details>
              <summary className="cursor-pointer text-xs font-medium">
                Previous runs ({qualityRuns.length - 1})
              </summary>
              <div className="mt-2 divide-y rounded-md border px-3">
                {qualityRuns.slice(1).map((run) => (
                  <div key={run.id} className="flex items-center gap-2 py-2">
                    <Badge variant={qualityStatusVariant(run.status)}>
                      {run.status}
                    </Badge>
                    <span>
                      {formatDate(run.completed_at ?? run.started_at)}
                    </span>
                    <span className="ml-auto text-muted-foreground">
                      {run.triggered_by}
                    </span>
                  </div>
                ))}
              </div>
            </details>
          ) : null}
        </CardContent>
      </Card>

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
