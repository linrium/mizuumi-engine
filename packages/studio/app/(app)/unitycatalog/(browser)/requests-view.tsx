"use client"

import { IconPlus, IconRefresh } from "@tabler/icons-react"
import { cn } from "cn"
import { useRouter } from "next/navigation"
import { useEffect, useState, useTransition } from "react"
import {
  cancelAccessRequest,
  createAccessRequest,
  updateAccessRequest,
} from "@/app/(app)/review/actions"
import type { AccessRequest } from "@/app/(app)/review/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
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

function formatDate(value?: string | null) {
  if (!value) return "—"
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value))
}

function statusVariant(
  status: string,
): "default" | "secondary" | "outline" | "destructive" {
  if (status === "APPROVED" || status === "FULFILLED") return "default"
  if (status === "REJECTED" || status === "FAILED") return "destructive"
  if (status === "AWAITING_APPROVAL") return "secondary"
  return "outline"
}

function canCancel(status: string) {
  return status === "AWAITING_APPROVAL"
}

const PRIVILEGES_BY_TYPE = {
  catalog: ["BROWSE", "USE CATALOG", "CREATE SCHEMA"],
  schema: [
    "BROWSE",
    "USE SCHEMA",
    "CREATE TABLE",
    "CREATE FUNCTION",
    "CREATE VOLUME",
    "CREATE MODEL",
  ],
  table: ["BROWSE", "SELECT", "MODIFY"],
} as const

type RequestSecurableType = keyof typeof PRIVILEGES_BY_TYPE

export function RequestsView({
  securableType,
  fullName,
  teams,
  initialRequests,
  initialError,
}: {
  securableType: RequestSecurableType
  fullName: string
  teams: string[]
  initialRequests: AccessRequest[]
  initialError?: string
}) {
  const privileges = PRIVILEGES_BY_TYPE[securableType]
  const router = useRouter()
  const [requests, setRequests] = useState(initialRequests)
  const [message, setMessage] = useState<{
    kind: "error" | "success"
    text: string
  } | null>(initialError ? { kind: "error", text: initialError } : null)
  const [createOpen, setCreateOpen] = useState(false)
  const [selected, setSelected] = useState<AccessRequest | null>(null)
  const [team, setTeam] = useState(teams[0] ?? "")
  const [privilege, setPrivilege] = useState<string>(privileges[0])
  const [durationHours, setDurationHours] = useState("8")
  const [reason, setReason] = useState("")
  const [isPending, startTransition] = useTransition()

  useEffect(() => setRequests(initialRequests), [initialRequests])

  function resetForm() {
    setTeam(teams[0] ?? "")
    setPrivilege(privileges[0])
    setDurationHours("8")
    setReason("")
  }

  function submitCreate() {
    const hours = Number(durationHours)
    if (!team || !Number.isInteger(hours) || hours <= 0 || !reason.trim()) {
      setMessage({
        kind: "error",
        text: "Team, reason, and a positive whole-hour duration are required.",
      })
      return
    }
    startTransition(async () => {
      const result = await createAccessRequest({
        routed_for_team: team,
        target_securable_type: securableType,
        target_full_name: fullName,
        requested_privilege: privilege,
        reason: reason.trim(),
        duration_hours: hours,
      })
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setRequests((items) => [result.data, ...items])
      setMessage({ kind: "success", text: "Access request submitted." })
      setCreateOpen(false)
      resetForm()
    })
  }

  function submitEdit() {
    if (!selected) return
    const hours = Number(durationHours)
    if (!Number.isInteger(hours) || hours <= 0 || !reason.trim()) {
      setMessage({
        kind: "error",
        text: "Reason and a positive whole-hour duration are required.",
      })
      return
    }
    startTransition(async () => {
      const result = await updateAccessRequest({
        requestId: selected.id,
        reason: reason.trim(),
        durationHours: hours,
      })
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setRequests((items) =>
        items.map((item) => (item.id === result.data.id ? result.data : item)),
      )
      setMessage({ kind: "success", text: "Access request updated." })
      setSelected(null)
    })
  }

  function openRequest(request: AccessRequest) {
    setSelected(request)
    setReason(request.reason)
    setDurationHours(String(request.requested_duration_seconds / 3600))
  }

  function cancelRequest(request: AccessRequest) {
    startTransition(async () => {
      const result = await cancelAccessRequest(request.id)
      if (!result.ok) {
        setMessage({ kind: "error", text: result.error })
        return
      }
      setRequests((items) =>
        items.map((item) => (item.id === result.data.id ? result.data : item)),
      )
      setMessage({ kind: "success", text: "Access request cancelled." })
      if (selected?.id === request.id) setSelected(result.data)
    })
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <div></div>
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="ghost"
            disabled={isPending}
            onClick={() => startTransition(() => router.refresh())}
          >
            <IconRefresh className={cn(isPending && "animate-spin")} />
            Reload
          </Button>
          <Button size="sm" onClick={() => setCreateOpen(true)}>
            <IconPlus /> Request
          </Button>
        </div>
      </div>

      {message ? (
        <div
          className={cn(
            "rounded border px-3 py-2 text-xs",
            message.kind === "error"
              ? "border-destructive/40 text-destructive"
              : "border-emerald-500/40 text-emerald-700 dark:text-emerald-400",
          )}
        >
          {message.text}
        </div>
      ) : null}

      <div className="overflow-hidden rounded-md ring-1 ring-foreground/10">
        <Table>
          <TableHeader className="bg-muted/50 text-muted-foreground">
            <TableRow>
              <TableHead className="h-auto px-3 py-2 text-muted-foreground">
                Policy
              </TableHead>
              <TableHead className="h-auto px-3 py-2 text-muted-foreground">
                Status
              </TableHead>
              <TableHead className="h-auto px-3 py-2 text-muted-foreground">
                Privileges
              </TableHead>
              <TableHead className="h-auto px-3 py-2 text-muted-foreground">
                Duration
              </TableHead>
              <TableHead className="h-auto px-3 py-2 text-muted-foreground">
                Expires
              </TableHead>
              <TableHead className="h-auto w-0 px-3 py-2" />
            </TableRow>
          </TableHeader>
          <TableBody>
            {requests.length === 0 ? (
              <TableRow className="border-b-0">
                <TableCell
                  colSpan={6}
                  className="px-3 py-6 text-center text-muted-foreground"
                >
                  No requests for this {securableType}.
                </TableCell>
              </TableRow>
            ) : (
              requests.map((request, index) => (
                <TableRow
                  key={request.id}
                  className={cn(
                    "cursor-pointer border-t border-b-0 hover:bg-muted/30",
                    index % 2 === 1 && "bg-muted/20",
                  )}
                  onClick={() => openRequest(request)}
                >
                  <TableCell className="px-3 py-1.5 font-medium">
                    {request.template_name}
                  </TableCell>
                  <TableCell className="px-3 py-1.5">
                    <Badge variant={statusVariant(request.status)}>
                      {request.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="px-3 py-1.5">
                    <div className="flex flex-wrap gap-1">
                      {request.grants
                        .filter((grant) => grant.full_name === fullName)
                        .map((grant) => (
                          <Badge key={grant.privilege} variant="secondary">
                            {grant.privilege}
                          </Badge>
                        ))}
                    </div>
                  </TableCell>
                  <TableCell className="px-3 py-1.5 text-muted-foreground">
                    {request.requested_duration_seconds / 3600}h
                  </TableCell>
                  <TableCell className="px-3 py-1.5 text-muted-foreground">
                    {formatDate(request.access_expires_at)}
                  </TableCell>
                  <TableCell className="px-3 py-1.5 text-right">
                    {canCancel(request.status) ? (
                      <Button
                        size="xs"
                        variant="outline"
                        disabled={isPending}
                        onClick={(event) => {
                          event.stopPropagation()
                          cancelRequest(request)
                        }}
                      >
                        Cancel
                      </Button>
                    ) : null}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog
        open={createOpen}
        onOpenChange={(open) => {
          setCreateOpen(open)
          if (!open) resetForm()
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Request {securableType} access</DialogTitle>
            <DialogDescription>
              Request temporary access to{" "}
              <span className="font-mono">{fullName}</span>.
            </DialogDescription>
          </DialogHeader>
          <RequestFields
            teams={teams}
            team={team}
            setTeam={setTeam}
            privilege={privilege}
            setPrivilege={setPrivilege}
            privileges={privileges}
            durationHours={durationHours}
            setDurationHours={setDurationHours}
            reason={reason}
            setReason={setReason}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button disabled={isPending || !team} onClick={submitCreate}>
              Request
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={!!selected}
        onOpenChange={(open) => !open && setSelected(null)}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Access request</DialogTitle>
            <DialogDescription>
              {selected?.template_name} · {selected?.status}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid gap-2">
              <label className="text-xs font-medium" htmlFor="edit-duration">
                Duration (hours)
              </label>
              <Input
                id="edit-duration"
                type="number"
                min="1"
                step="1"
                disabled={selected?.status !== "AWAITING_APPROVAL"}
                value={durationHours}
                onChange={(event) => setDurationHours(event.target.value)}
              />
            </div>
            <div className="grid gap-2">
              <label className="text-xs font-medium" htmlFor="edit-reason">
                Reason
              </label>
              <Textarea
                id="edit-reason"
                rows={3}
                disabled={selected?.status !== "AWAITING_APPROVAL"}
                value={reason}
                onChange={(event) => setReason(event.target.value)}
              />
            </div>
            {selected?.status !== "AWAITING_APPROVAL" ? (
              <p className="text-xs text-muted-foreground">
                Only requests awaiting approval can be edited.
              </p>
            ) : null}
          </div>
          <DialogFooter>
            {selected && canCancel(selected.status) ? (
              <Button
                variant="outline"
                disabled={isPending}
                onClick={() => cancelRequest(selected)}
              >
                Cancel request
              </Button>
            ) : null}
            <Button
              disabled={isPending || selected?.status !== "AWAITING_APPROVAL"}
              onClick={submitEdit}
            >
              Save changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function RequestFields({
  teams,
  team,
  setTeam,
  privilege,
  setPrivilege,
  privileges,
  durationHours,
  setDurationHours,
  reason,
  setReason,
}: {
  teams: string[]
  team: string
  setTeam: (value: string) => void
  privilege: string
  setPrivilege: (value: string) => void
  privileges: readonly string[]
  durationHours: string
  setDurationHours: (value: string) => void
  reason: string
  setReason: (value: string) => void
}) {
  return (
    <div className="space-y-4">
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-team">
          Team
        </label>
        <Select
          items={teams.map((item) => ({ value: item, label: item }))}
          value={team}
          onValueChange={(value) => value && setTeam(value)}
        >
          <SelectTrigger id="request-team" className="w-full">
            <SelectValue placeholder="Select team" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {teams.map((item) => (
                <SelectItem key={item} value={item}>
                  {item}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-privilege">
          Privilege
        </label>
        <Select
          items={privileges.map((item) => ({ value: item, label: item }))}
          value={privilege}
          onValueChange={(value) => value && setPrivilege(value)}
        >
          <SelectTrigger id="request-privilege" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {privileges.map((item) => (
                <SelectItem key={item} value={item}>
                  {item}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-duration">
          Duration (hours)
        </label>
        <Input
          id="request-duration"
          type="number"
          min="1"
          step="1"
          value={durationHours}
          onChange={(event) => setDurationHours(event.target.value)}
        />
      </div>
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-reason">
          Reason
        </label>
        <Textarea
          id="request-reason"
          rows={3}
          value={reason}
          onChange={(event) => setReason(event.target.value)}
        />
      </div>
    </div>
  )
}
