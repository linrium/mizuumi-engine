"use client"

import { IconAlertTriangle, IconPlus, IconTrash } from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
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

const TEAM_ORGS: { org: string; teams: string[] }[] = [
  { org: "vietjet", teams: ["vietjetGrow"] },
  { org: "hdbank", teams: ["hdbankGrow"] },
]

const TEAM_ITEMS = TEAM_ORGS.flatMap(({ teams }) =>
  teams.map((team) => ({ value: team, label: team })),
)

const selectItems = (values: string[]) =>
  values.map((value) => ({ value, label: value }))

function formatDuration(seconds?: number | null) {
  if (!seconds) return "—"
  if (seconds % 86400 === 0) return `${seconds / 86400}d`
  if (seconds % 3600 === 0) return `${seconds / 3600}h`
  return `${seconds}s`
}

export function PolicyTemplates({
  initialTemplates,
  initialError,
}: {
  initialTemplates: PolicyTemplate[]
  initialError?: string
}) {
  const router = useRouter()
  const [dialogOpen, setDialogOpen] = useState(false)
  const [key, setKey] = useState("")
  const [name, setName] = useState("")
  const [description, setDescription] = useState("")
  const [affectedTeams, setAffectedTeams] = useState<string[]>([])
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

  function resetForm() {
    setKey("")
    setName("")
    setDescription("")
    setAffectedTeams([])
    setApprovalStrategy("ANY")
    setDefaultDurationHours("8")
    setMaxDurationHours("24")
    setApprovalTeams([{ rowId: 1, team: "", required_approvals: 1 }])
    setGrants([
      {
        rowId: 1,
        securable_type: "table",
        target_kind: "REQUEST_RESOURCE",
        target_full_name: "",
        privilege: "SELECT",
      },
    ])
    setNextRowId(2)
    setMessage(null)
  }

  function toggleAffectedTeam(team: string) {
    setAffectedTeams((teams) =>
      teams.includes(team)
        ? teams.filter((item) => item !== team)
        : [...teams, team],
    )
  }

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
    const parsedApprovalTeams = approvalTeams
      .map(({ team, required_approvals }) => ({
        team: team.trim(),
        required_approvals,
      }))
      .filter((team) => team.team)
    const defaultDuration = Number(defaultDurationHours) * 3600
    const maxDuration = Number(maxDurationHours) * 3600

    if (!key.trim() || !name.trim()) {
      setMessage({ kind: "error", text: "Key and name are required." })
      return
    }
    if (affectedTeams.length === 0 || parsedApprovalTeams.length === 0) {
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
        affected_teams: affectedTeams,
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
      router.refresh()
      setDialogOpen(false)
      resetForm()
    })
  }

  return (
    <section className="w-full space-y-4">
      {initialError ? (
        <div className="flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
          <IconAlertTriangle className="size-4 shrink-0" />
          {initialError}
        </div>
      ) : null}

      <div className="flex justify-end">
        <Dialog
          open={dialogOpen}
          onOpenChange={(open) => {
            if (!open && isCreating) return
            setDialogOpen(open)
            if (!open) resetForm()
          }}
        >
          <DialogTrigger
            render={
              <Button size="sm">
                <IconPlus />
                New policy template
              </Button>
            }
          />
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Create policy template</DialogTitle>
              <DialogDescription>
                New templates are published immediately as version 1.
              </DialogDescription>
            </DialogHeader>

            <div className="max-h-[65vh] space-y-4 overflow-y-auto pr-1">
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
                <p className="text-xs text-muted-foreground">
                  Stable machine identifier used by access requests. Lowercase
                  and kebab-case; cannot be changed after publishing.
                </p>
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
                <p className="text-xs text-muted-foreground">
                  Human-readable title shown to requesters and reviewers.
                </p>
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
                <p className="text-xs text-muted-foreground">
                  Explain when this policy should be used so requesters pick the
                  right one.
                </p>
              </div>

              <div className="grid gap-2">
                <span className="text-xs font-medium">Affected teams</span>
                <Select
                  items={TEAM_ITEMS}
                  value=""
                  onValueChange={(value) => value && toggleAffectedTeam(value)}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Add affected team" />
                  </SelectTrigger>
                  <SelectContent>
                    {TEAM_ORGS.map(({ org, teams }) => (
                      <SelectGroup key={org}>
                        <SelectLabel>{org}</SelectLabel>
                        {teams.map((team) => (
                          <SelectItem key={team} value={team}>
                            {team}
                          </SelectItem>
                        ))}
                      </SelectGroup>
                    ))}
                  </SelectContent>
                </Select>
                {affectedTeams.length > 0 ? (
                  <div className="flex flex-wrap gap-1">
                    {affectedTeams.map((team) => (
                      <Badge key={team} variant="secondary" className="gap-1">
                        {team}
                        <button
                          type="button"
                          onClick={() => toggleAffectedTeam(team)}
                          aria-label={`Remove ${team}`}
                          className="text-muted-foreground hover:text-foreground"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                ) : null}
                <p className="text-xs text-muted-foreground">
                  Teams whose members are allowed to submit a request using this
                  policy.
                </p>
              </div>

              <div className="grid gap-2">
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
                    <label
                      className="text-xs font-medium"
                      htmlFor="max-duration"
                    >
                      Maximum hours
                    </label>
                    <Input
                      id="max-duration"
                      type="number"
                      min="1"
                      value={maxDurationHours}
                      onChange={(event) =>
                        setMaxDurationHours(event.target.value)
                      }
                    />
                  </div>
                </div>
                <p className="text-xs text-muted-foreground">
                  How long granted access lasts. Default is pre-filled on the
                  request; maximum caps what a requester can ask for.
                </p>
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
                <p className="text-xs text-muted-foreground">
                  <strong>Any</strong> approves once one team hits its quorum.{" "}
                  <strong>Every</strong> requires every listed team to hit its
                  quorum.
                </p>
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
                <p className="text-xs text-muted-foreground">
                  Teams that review requests, with the number of approvals each
                  team must collect to reach quorum.
                </p>
                {approvalTeams.map((team) => (
                  <div key={team.rowId} className="flex gap-2">
                    <Select
                      items={TEAM_ITEMS}
                      value={team.team}
                      onValueChange={(value) =>
                        value &&
                        setApprovalTeams((teams) =>
                          teams.map((item) =>
                            item.rowId === team.rowId
                              ? { ...item, team: value }
                              : item,
                          ),
                        )
                      }
                    >
                      <SelectTrigger
                        className="w-full"
                        aria-label="Approval team"
                      >
                        <SelectValue placeholder="Select team" />
                      </SelectTrigger>
                      <SelectContent>
                        {TEAM_ORGS.map(({ org, teams }) => (
                          <SelectGroup key={org}>
                            <SelectLabel>{org}</SelectLabel>
                            {teams.map((item) => (
                              <SelectItem key={item} value={item}>
                                {item}
                              </SelectItem>
                            ))}
                          </SelectGroup>
                        ))}
                      </SelectContent>
                    </Select>
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
                <p className="text-xs text-muted-foreground">
                  Grants issued when a request is approved. Pick{" "}
                  <strong>Requested resource</strong> to apply to whatever the
                  requester targets, or <strong>Fixed resource</strong> to
                  always grant on a specific full name.
                </p>
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
                        <SelectTrigger
                          className="w-full"
                          aria-label="Privilege"
                        >
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
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                disabled={isCreating}
                onClick={() => {
                  setDialogOpen(false)
                  resetForm()
                }}
              >
                Cancel
              </Button>
              <Button disabled={isCreating} onClick={submitTemplate}>
                <IconPlus />
                {isCreating ? "Publishing…" : "Create and publish"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="pl-4">Template</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Strategy</TableHead>
              <TableHead>Default / Max</TableHead>
              <TableHead>Approval teams</TableHead>
              <TableHead>Affected teams</TableHead>
              <TableHead className="pr-4">Privileges</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialTemplates.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  No policy templates.
                </TableCell>
              </TableRow>
            ) : (
              initialTemplates.map((template) => (
                <TableRow key={template.id}>
                  <TableCell className="pl-4">
                    <div className="flex flex-col">
                      <span className="font-medium">{template.name}</span>
                      <span className="font-mono text-muted-foreground">
                        {template.key} · v{template.version}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge>{template.status}</Badge>
                  </TableCell>
                  <TableCell>
                    <Badge variant="secondary">
                      {template.approval_strategy}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    {formatDuration(template.default_duration_seconds)} /{" "}
                    {formatDuration(template.max_duration_seconds)}
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {template.approval_teams.map((team) => (
                        <Badge key={team.team} variant="outline">
                          {team.team} ({team.required_approvals})
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {template.affected_teams.map((team) => (
                        <Badge key={team} variant="outline">
                          {team}
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell className="pr-4">
                    <div className="flex flex-wrap gap-1">
                      {template.grants.map((grant) => (
                        <Badge
                          key={`${grant.securable_type}:${grant.target_kind}:${grant.target_full_name}:${grant.privilege}`}
                          variant="outline"
                        >
                          {grant.securable_type} · {grant.privilege}
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </section>
  )
}
