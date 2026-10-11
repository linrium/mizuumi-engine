"use client"

import {
  IconAlertTriangle,
  IconClock,
  IconPlus,
  IconRoute,
  IconShieldCheck,
  IconTemplate,
  IconTrash,
} from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { createPolicyTemplate } from "../actions"
import type { ApprovalTeam, PolicyGrant, PolicyTemplate } from "../types"

type DraftApprovalTeam = ApprovalTeam & { rowId: number }
type DraftGrant = PolicyGrant & { rowId: number }

const SECURABLE_TYPES = [
  "metastore",
  "catalog",
  "schema",
  "table",
  "function",
  "volume",
  "registered_model",
  "external_location",
  "credential",
]

const PRIVILEGES = [
  "OWNER",
  "CREATE CATALOG",
  "USE CATALOG",
  "CREATE SCHEMA",
  "USE SCHEMA",
  "CREATE TABLE",
  "SELECT",
  "MODIFY",
  "CREATE FUNCTION",
  "EXECUTE",
  "CREATE VOLUME",
  "READ VOLUME",
  "CREATE MODEL",
  "CREATE EXTERNAL LOCATION",
  "READ FILES",
  "WRITE FILES",
  "CREATE EXTERNAL TABLE",
  "CREATE EXTERNAL VOLUME",
  "CREATE MANAGED STORAGE",
  "CREATE STORAGE CREDENTIAL",
]

const selectItems = (values: string[]) =>
  values.map((value) => ({ value, label: value }))

function formatDuration(seconds?: number | null) {
  if (!seconds) return "Not set"
  if (seconds % 86400 === 0) return `${seconds / 86400} days`
  if (seconds % 3600 === 0) return `${seconds / 3600} hours`
  return `${seconds} seconds`
}

function normalizeTeamName(value: string) {
  return value.trim().replace(/^\/+|\/+$/g, "")
}

function splitTeams(value: string) {
  return [...new Set(value.split(",").map(normalizeTeamName).filter(Boolean))]
}

export function PolicyTemplates({
  initialTemplates,
  initialError,
}: {
  initialTemplates: PolicyTemplate[]
  initialError?: string
}) {
  const router = useRouter()
  const [key, setKey] = useState("")
  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [affectedTeams, setAffectedTeams] = useState("")
  const [approvalStrategy, setApprovalStrategy] = useState<"ANY" | "ALL">("ANY")
  const [defaultDurationHours, setDefaultDurationHours] = useState("8")
  const [maxDurationHours, setMaxDurationHours] = useState("24")
  const [approvalTeams, setApprovalTeams] = useState<DraftApprovalTeam[]>([
    { rowId: 1, team: "", required_approvals: 1 },
  ])
  const [grants, setGrants] = useState<DraftGrant[]>([
    {
      rowId: 1,
      securable_type: "table",
      target_kind: "REQUEST_RESOURCE",
      target_full_name: "",
      privilege: "SELECT",
    },
  ])
  const [nextRowId, setNextRowId] = useState(2)
  const [message, setMessage] = useState<{
    kind: "success" | "error"
    text: string
  } | null>(null)
  const [isCreating, startCreating] = useTransition()

  function addApprovalTeam() {
    setApprovalTeams((teams) => [
      ...teams,
      { rowId: nextRowId, team: "", required_approvals: 1 },
    ])
    setNextRowId((id) => id + 1)
  }

  function addGrant() {
    setGrants((items) => [
      ...items,
      {
        rowId: nextRowId,
        securable_type: "table",
        target_kind: "REQUEST_RESOURCE",
        target_full_name: "",
        privilege: "SELECT",
      },
    ])
    setNextRowId((id) => id + 1)
  }

  function submitTemplate() {
    setMessage(null)
    const parsedAffectedTeams = splitTeams(affectedTeams)
    const parsedApprovalTeams = approvalTeams
      .map(({ team, required_approvals }) => ({
        team: normalizeTeamName(team),
        required_approvals,
      }))
      .filter((team) => team.team)
    const defaultDuration = Number(defaultDurationHours) * 3600
    const maxDuration = Number(maxDurationHours) * 3600

    if (!key.trim() || !name.trim()) {
      setMessage({ kind: "error", text: "Key and name are required." })
      return
    }
    if (parsedAffectedTeams.length === 0 || parsedApprovalTeams.length === 0) {
      setMessage({
        kind: "error",
        text: "Add at least one affected team and one approval team.",
      })
      return
    }
    if (!Number.isFinite(defaultDuration) || !Number.isFinite(maxDuration)) {
      setMessage({ kind: "error", text: "Durations must be valid numbers." })
      return
    }
    if (
      defaultDuration <= 0 ||
      maxDuration <= 0 ||
      defaultDuration > maxDuration
    ) {
      setMessage({
        kind: "error",
        text: "Durations must be positive and default cannot exceed maximum.",
      })
      return
    }

    startCreating(async () => {
      const result = await createPolicyTemplate({
        key: key.trim(),
        name: name.trim(),
        description: description.trim() || undefined,
        approval_strategy: approvalStrategy,
        default_duration_seconds: defaultDuration,
        max_duration_seconds: maxDuration,
        affected_teams: parsedAffectedTeams,
        approval_teams: parsedApprovalTeams,
        grants: grants.map(
          ({ rowId: _rowId, ...grant }): PolicyGrant => ({
            ...grant,
            target_full_name: grant.target_full_name.trim(),
          }),
        ),
      })
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setMessage({ kind: "success", text: `Published ${result.data.name}.` })
      setKey("")
      setName("")
      setDescription("")
      router.refresh()
    })
  }

  return (
    <section className="space-y-5">
      <div>
        <h2 className="text-lg font-semibold">Policy templates</h2>
        <p className="text-sm text-muted-foreground">
          Package privileges, routing, and duration limits into reusable access
          policies.
        </p>
      </div>

      {initialError ? (
        <div className="flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
          <IconAlertTriangle className="size-4 shrink-0" />
          {initialError}
        </div>
      ) : null}

      <div className="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_26rem]">
        <div className="space-y-4">
          {initialTemplates.length === 0 ? (
            <Card>
              <CardHeader className="items-center py-10 text-center">
                <IconTemplate className="size-8 text-muted-foreground" />
                <CardTitle>No policy templates yet</CardTitle>
                <CardDescription>
                  Create the first template to standardize an access request.
                </CardDescription>
              </CardHeader>
            </Card>
          ) : (
            initialTemplates.map((template) => (
              <Card key={template.id}>
                <CardHeader className="border-b">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <div className="mb-2 flex items-center gap-2">
                        <Badge>{template.status}</Badge>
                        <Badge variant="outline">v{template.version}</Badge>
                        <span className="font-mono text-xs text-muted-foreground">
                          {template.key}
                        </span>
                      </div>
                      <CardTitle className="text-base">
                        {template.name}
                      </CardTitle>
                      <CardDescription className="mt-1 max-w-2xl">
                        {template.description || "No description provided."}
                      </CardDescription>
                    </div>
                    <Badge variant="secondary">
                      {template.approval_strategy} approval
                    </Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="grid gap-3 sm:grid-cols-3">
                    <div className="flex items-start gap-2">
                      <IconClock className="mt-0.5 size-4 text-muted-foreground" />
                      <div>
                        <div className="text-muted-foreground">
                          Default / maximum
                        </div>
                        <div className="font-medium">
                          {formatDuration(template.default_duration_seconds)} /{" "}
                          {formatDuration(template.max_duration_seconds)}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-start gap-2">
                      <IconRoute className="mt-0.5 size-4 text-muted-foreground" />
                      <div>
                        <div className="text-muted-foreground">
                          Approval teams
                        </div>
                        <div className="font-medium">
                          {template.approval_teams
                            .map(
                              (team) =>
                                `${team.team} (${team.required_approvals})`,
                            )
                            .join(", ")}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-start gap-2">
                      <IconShieldCheck className="mt-0.5 size-4 text-muted-foreground" />
                      <div>
                        <div className="text-muted-foreground">
                          Affected teams
                        </div>
                        <div className="font-medium">
                          {template.affected_teams.join(", ")}
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {template.grants.map((grant) => (
                      <Badge
                        key={`${grant.securable_type}:${grant.target_kind}:${grant.target_full_name}:${grant.privilege}`}
                        variant="outline"
                      >
                        {grant.securable_type} · {grant.privilege}
                        {grant.target_kind === "FIXED_RESOURCE"
                          ? ` · ${grant.target_full_name}`
                          : " · requested resource"}
                      </Badge>
                    ))}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Created by {template.created_by}
                  </p>
                </CardContent>
              </Card>
            ))
          )}
        </div>

        <Card className="xl:sticky xl:top-6">
          <CardHeader className="border-b">
            <CardTitle>Create policy template</CardTitle>
            <CardDescription>
              New templates are published immediately as version 1.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-2">
              <label className="text-xs font-medium" htmlFor="policy-key">
                Key
              </label>
              <Input
                id="policy-key"
                value={key}
                onChange={(event) => setKey(event.target.value)}
                placeholder="analytics-read"
              />
            </div>
            <div className="grid gap-2">
              <label className="text-xs font-medium" htmlFor="policy-name">
                Name
              </label>
              <Input
                id="policy-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="Analytics read access"
              />
            </div>
            <div className="grid gap-2">
              <label
                className="text-xs font-medium"
                htmlFor="policy-description"
              >
                Description
              </label>
              <Textarea
                id="policy-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder="When this policy should be used"
                rows={2}
              />
            </div>
            <div className="grid gap-2">
              <label className="text-xs font-medium" htmlFor="affected-teams">
                Affected teams
              </label>
              <Input
                id="affected-teams"
                value={affectedTeams}
                onChange={(event) => setAffectedTeams(event.target.value)}
                placeholder="analytics, finance"
              />
              <p className="text-xs text-muted-foreground">Comma separated.</p>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="grid gap-2">
                <label
                  className="text-xs font-medium"
                  htmlFor="default-duration"
                >
                  Default hours
                </label>
                <Input
                  id="default-duration"
                  type="number"
                  min="1"
                  value={defaultDurationHours}
                  onChange={(event) =>
                    setDefaultDurationHours(event.target.value)
                  }
                />
              </div>
              <div className="grid gap-2">
                <label className="text-xs font-medium" htmlFor="max-duration">
                  Maximum hours
                </label>
                <Input
                  id="max-duration"
                  type="number"
                  min="1"
                  value={maxDurationHours}
                  onChange={(event) => setMaxDurationHours(event.target.value)}
                />
              </div>
            </div>

            <div className="grid gap-2">
              <label
                className="text-xs font-medium"
                htmlFor="approval-strategy"
              >
                Approval strategy
              </label>
              <Select
                items={selectItems(["ANY", "ALL"])}
                value={approvalStrategy}
                onValueChange={(value) =>
                  value && setApprovalStrategy(value as "ANY" | "ALL")
                }
              >
                <SelectTrigger id="approval-strategy" className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value="ANY">Any team quorum</SelectItem>
                    <SelectItem value="ALL">Every team quorum</SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-medium">Approval teams</span>
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  onClick={addApprovalTeam}
                >
                  <IconPlus /> Add
                </Button>
              </div>
              {approvalTeams.map((team) => (
                <div key={team.rowId} className="flex gap-2">
                  <Input
                    aria-label="Approval team"
                    value={team.team}
                    onChange={(event) =>
                      setApprovalTeams((teams) =>
                        teams.map((item) =>
                          item.rowId === team.rowId
                            ? { ...item, team: event.target.value }
                            : item,
                        ),
                      )
                    }
                    placeholder="data-stewards"
                  />
                  <Input
                    aria-label="Required approvals"
                    className="w-20"
                    type="number"
                    min="1"
                    value={team.required_approvals}
                    onChange={(event) =>
                      setApprovalTeams((teams) =>
                        teams.map((item) =>
                          item.rowId === team.rowId
                            ? {
                                ...item,
                                required_approvals: Math.max(
                                  1,
                                  Number(event.target.value),
                                ),
                              }
                            : item,
                        ),
                      )
                    }
                  />
                  <Button
                    type="button"
                    size="icon-sm"
                    variant="ghost"
                    disabled={approvalTeams.length === 1}
                    onClick={() =>
                      setApprovalTeams((teams) =>
                        teams.filter((item) => item.rowId !== team.rowId),
                      )
                    }
                    aria-label="Remove approval team"
                  >
                    <IconTrash />
                  </Button>
                </div>
              ))}
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-xs font-medium">Privileges</span>
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  onClick={addGrant}
                >
                  <IconPlus /> Add
                </Button>
              </div>
              {grants.map((grant) => (
                <div
                  key={grant.rowId}
                  className="space-y-2 rounded-md border p-2"
                >
                  <div className="grid grid-cols-2 gap-2">
                    <Select
                      items={selectItems(SECURABLE_TYPES)}
                      value={grant.securable_type}
                      onValueChange={(value) =>
                        value &&
                        setGrants((items) =>
                          items.map((item) =>
                            item.rowId === grant.rowId
                              ? { ...item, securable_type: value }
                              : item,
                          ),
                        )
                      }
                    >
                      <SelectTrigger
                        className="w-full"
                        aria-label="Securable type"
                      >
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectGroup>
                          {SECURABLE_TYPES.map((type) => (
                            <SelectItem key={type} value={type}>
                              {type}
                            </SelectItem>
                          ))}
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                    <Select
                      items={selectItems(PRIVILEGES)}
                      value={grant.privilege}
                      onValueChange={(value) =>
                        value &&
                        setGrants((items) =>
                          items.map((item) =>
                            item.rowId === grant.rowId
                              ? { ...item, privilege: value }
                              : item,
                          ),
                        )
                      }
                    >
                      <SelectTrigger className="w-full" aria-label="Privilege">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectGroup>
                          {PRIVILEGES.map((privilege) => (
                            <SelectItem key={privilege} value={privilege}>
                              {privilege}
                            </SelectItem>
                          ))}
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex gap-2">
                    <Select
                      items={selectItems([
                        "REQUEST_RESOURCE",
                        "FIXED_RESOURCE",
                      ])}
                      value={grant.target_kind}
                      onValueChange={(value) =>
                        value &&
                        setGrants((items) =>
                          items.map((item) =>
                            item.rowId === grant.rowId
                              ? {
                                  ...item,
                                  target_kind:
                                    value as PolicyGrant["target_kind"],
                                  target_full_name:
                                    value === "REQUEST_RESOURCE"
                                      ? ""
                                      : item.target_full_name,
                                }
                              : item,
                          ),
                        )
                      }
                    >
                      <SelectTrigger
                        className="w-full"
                        aria-label="Target kind"
                      >
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectGroup>
                          <SelectItem value="REQUEST_RESOURCE">
                            Requested resource
                          </SelectItem>
                          <SelectItem value="FIXED_RESOURCE">
                            Fixed resource
                          </SelectItem>
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                    <Button
                      type="button"
                      size="icon-sm"
                      variant="ghost"
                      disabled={grants.length === 1}
                      onClick={() =>
                        setGrants((items) =>
                          items.filter((item) => item.rowId !== grant.rowId),
                        )
                      }
                      aria-label="Remove privilege"
                    >
                      <IconTrash />
                    </Button>
                  </div>
                  {grant.target_kind === "FIXED_RESOURCE" ? (
                    <Input
                      aria-label="Fixed resource full name"
                      value={grant.target_full_name}
                      onChange={(event) =>
                        setGrants((items) =>
                          items.map((item) =>
                            item.rowId === grant.rowId
                              ? {
                                  ...item,
                                  target_full_name: event.target.value,
                                }
                              : item,
                          ),
                        )
                      }
                      placeholder="main.analytics.orders"
                    />
                  ) : null}
                </div>
              ))}
            </div>

            {message ? (
              <p
                className={
                  message.kind === "error"
                    ? "text-xs text-destructive"
                    : "text-xs text-green-700"
                }
              >
                {message.text}
              </p>
            ) : null}

            <Button
              className="w-full"
              disabled={isCreating}
              onClick={submitTemplate}
            >
              <IconPlus />
              {isCreating ? "Publishing…" : "Create and publish"}
            </Button>
          </CardContent>
        </Card>
      </div>
    </section>
  )
}
