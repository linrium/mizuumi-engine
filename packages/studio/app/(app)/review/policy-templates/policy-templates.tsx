"use client"

import { IconAlertTriangle, IconPlus, IconTrash } from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Combobox,
  ComboboxChip,
  ComboboxChips,
  ComboboxChipsInput,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  ComboboxValue,
} from "@/components/ui/combobox"
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

const PRIVILEGES_BY_TYPE: Record<string, string[]> = {
  metastore: ["CREATE CATALOG"],
  catalog: ["USE CATALOG", "CREATE SCHEMA"],
  schema: [
    "USE SCHEMA",
    "CREATE TABLE",
    "CREATE FUNCTION",
    "CREATE VOLUME",
    "CREATE MODEL",
  ],
  table: ["SELECT", "MODIFY"],
  function: ["EXECUTE"],
  volume: ["READ VOLUME"],
  registered_model: ["EXECUTE"],
  external_location: [
    "READ FILES",
    "WRITE FILES",
    "CREATE EXTERNAL TABLE",
    "CREATE EXTERNAL VOLUME",
    "CREATE MANAGED STORAGE",
  ],
  credential: ["CREATE EXTERNAL LOCATION", "CREATE STORAGE CREDENTIAL"],
}

const TEAM_ORGS: { org: string; teams: string[] }[] = [
  { org: "vietjet", teams: ["vietjetGrow"] },
  { org: "hdbank", teams: ["hdbankGrow"] },
  { org: "sovico", teams: ["sovicoSecurity"] },
]

const TEAM_ITEMS = TEAM_ORGS.flatMap(({ teams }) =>
  teams.map((team) => ({ value: team, label: team })),
)
const TEAM_NAMES = TEAM_ITEMS.map((item) => item.value)

const selectItems = (values: string[]) =>
  values.map((value) => ({ value, label: value }))

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
  const [autoApprove, setAutoApprove] = useState(false)
  const [approvalTeams, setApprovalTeams] = useState<DraftApprovalTeam[]>([
    { rowId: 1, team: "", required_approvals: 1 },
  ])
  const [grant, setGrant] = useState<PolicyGrant>({
    securable_type: "table",
    target_full_name: "",
    privilege: "SELECT",
  })
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
    setAutoApprove(false)
    setApprovalTeams([{ rowId: 1, team: "", required_approvals: 1 }])
    setGrant({
      securable_type: "table",
      target_full_name: "",
      privilege: "SELECT",
    })
    setNextRowId(2)
    setMessage(null)
  }

  function addApprovalTeam() {
    setApprovalTeams((teams) => [
      ...teams,
      { rowId: nextRowId, team: "", required_approvals: 1 },
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

    if (!key.trim() || !name.trim()) {
      setMessage({ kind: "error", text: "Key and name are required." })
      return
    }
    if (
      affectedTeams.length === 0 ||
      (!autoApprove && parsedApprovalTeams.length === 0)
    ) {
      setMessage({
        kind: "error",
        text: autoApprove
          ? "Add at least one affected team."
          : "Add at least one affected team and one approval team.",
      })
      return
    }
    if (!grant.target_full_name.trim()) {
      setMessage({
        kind: "error",
        text: "A fixed securable full name is required.",
      })
      return
    }

    startCreating(async () => {
      const result = await createPolicyTemplate({
        key: key.trim(),
        name: name.trim(),
        description: description.trim() || undefined,
        approval_strategy: approvalStrategy,
        auto_approve: autoApprove,
        affected_teams: affectedTeams,
        approval_teams: autoApprove ? [] : parsedApprovalTeams,
        grant: {
          ...grant,
          target_full_name: grant.target_full_name.trim(),
        },
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
                  Explain the policy scope and intent for reviewers.
                </p>
              </div>

              <div className="grid gap-2">
                <label className="text-xs font-medium" htmlFor="affected-team">
                  Affected teams
                </label>
                <Combobox
                  items={TEAM_NAMES}
                  multiple
                  value={affectedTeams}
                  onValueChange={setAffectedTeams}
                >
                  <ComboboxChips>
                    <ComboboxValue>
                      {affectedTeams.map((team) => (
                        <ComboboxChip key={team}>{team}</ComboboxChip>
                      ))}
                    </ComboboxValue>
                    <ComboboxChipsInput
                      id="affected-team"
                      placeholder="Add affected team"
                    />
                  </ComboboxChips>
                  <ComboboxContent>
                    <ComboboxEmpty>No teams found.</ComboboxEmpty>
                    <ComboboxList>
                      {(team) => (
                        <ComboboxItem key={team} value={team}>
                          {team}
                        </ComboboxItem>
                      )}
                    </ComboboxList>
                  </ComboboxContent>
                </Combobox>
                <p className="text-xs text-muted-foreground">
                  Teams whose members are allowed to submit a request using this
                  policy.
                </p>
              </div>

              <label className="flex items-start gap-3 rounded-md border p-3">
                <input
                  type="checkbox"
                  className="mt-0.5 size-4"
                  checked={autoApprove}
                  onChange={(event) => setAutoApprove(event.target.checked)}
                />
                <span className="grid gap-1">
                  <span className="text-xs font-medium">Auto approve</span>
                  <span className="text-xs text-muted-foreground">
                    Immediately approve requests matching this fixed securable
                    and privilege.
                  </span>
                </span>
              </label>

              {!autoApprove ? (
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
              ) : null}

              {!autoApprove ? (
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
                    Teams that review requests, with the number of approvals
                    each team must collect to reach quorum.
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
              ) : null}

              <div className="space-y-2">
                <span className="text-xs font-medium">Fixed privilege</span>
                <p className="text-xs text-muted-foreground">
                  Each template grants exactly one privilege on one fixed
                  securable.
                </p>
                <div className="space-y-2 rounded-md border p-2">
                  <div className="grid grid-cols-2 gap-2">
                    <Select
                      items={selectItems(SECURABLE_TYPES)}
                      value={grant.securable_type}
                      onValueChange={(value) => {
                        if (!value) return
                        setGrant({
                          ...grant,
                          securable_type: value,
                          privilege: PRIVILEGES_BY_TYPE[value][0],
                        })
                      }}
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
                    <Combobox
                      items={PRIVILEGES_BY_TYPE[grant.securable_type]}
                      value={grant.privilege}
                      onValueChange={(value) => {
                        if (!value) return
                        setGrant({ ...grant, privilege: value })
                      }}
                    >
                      <ComboboxInput
                        className="w-full"
                        aria-label="Privilege"
                        placeholder="Select privilege"
                      />
                      <ComboboxContent>
                        <ComboboxEmpty>No privileges found.</ComboboxEmpty>
                        <ComboboxList>
                          {(privilege) => (
                            <ComboboxItem key={privilege} value={privilege}>
                              {privilege}
                            </ComboboxItem>
                          )}
                        </ComboboxList>
                      </ComboboxContent>
                    </Combobox>
                  </div>
                  <Input
                    aria-label="Fixed resource full name"
                    value={grant.target_full_name}
                    onChange={(event) =>
                      setGrant({
                        ...grant,
                        target_full_name: event.target.value,
                      })
                    }
                    placeholder="main.analytics.orders"
                  />
                </div>
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
              <TableHead>Approval</TableHead>
              <TableHead>Approval teams</TableHead>
              <TableHead>Affected teams</TableHead>
              <TableHead className="pr-4">Privileges</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialTemplates.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
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
                      {template.auto_approve
                        ? "Automatic"
                        : `Manual · ${template.approval_strategy}`}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {template.auto_approve ? (
                        <span className="text-muted-foreground">
                          Not required
                        </span>
                      ) : (
                        template.approval_teams.map((team) => (
                          <Badge key={team.team} variant="outline">
                            {team.team} ({team.required_approvals})
                          </Badge>
                        ))
                      )}
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
                    <Badge variant="outline">
                      {template.grant.securable_type} ·{" "}
                      {template.grant.privilege}
                    </Badge>
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
