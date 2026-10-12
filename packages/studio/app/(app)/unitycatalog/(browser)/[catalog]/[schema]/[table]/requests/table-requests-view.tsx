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

const TABLE_PRIVILEGES = ["SELECT", "MODIFY"]

export function TableRequestsView({
  fullName,
  teams,
  initialRequests,
  initialError,
}: {
  fullName: string
  teams: string[]
  initialRequests: AccessRequest[]
  initialError?: string
}) {
  const router = useRouter()
  const [requests, setRequests] = useState(initialRequests)
  const [message, setMessage] = useState<{
    kind: "error" | "success"
    text: string
  } | null>(initialError ? { kind: "error", text: initialError } : null)
  const [createOpen, setCreateOpen] = useState(false)
  const [selected, setSelected] = useState<AccessRequest | null>(null)
  const [team, setTeam] = useState(teams[0] ?? "")
  const [privilege, setPrivilege] = useState(TABLE_PRIVILEGES[0])
  const [durationHours, setDurationHours] = useState("8")
  const [reason, setReason] = useState("")
  const [isPending, startTransition] = useTransition()

  useEffect(() => setRequests(initialRequests), [initialRequests])

  function resetForm() {
    setTeam(teams[0] ?? "")
    setPrivilege(TABLE_PRIVILEGES[0])
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
        target_securable_type: "table",
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
        <div>
          <div className="text-sm font-medium">Access requests</div>
          <div className="font-mono text-xs text-muted-foreground">
            {fullName}
          </div>
        </div>
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

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Policy</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Privileges</TableHead>
            <TableHead>Duration</TableHead>
            <TableHead>Expires</TableHead>
            <TableHead className="w-0" />
          </TableRow>
        </TableHeader>
        <TableBody>
          {requests.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={6}
                className="py-6 text-center text-muted-foreground"
              >
                No requests for this table.
              </TableCell>
            </TableRow>
          ) : (
            requests.map((request) => (
              <TableRow
                key={request.id}
                className="cursor-pointer"
                onClick={() => openRequest(request)}
              >
                <TableCell>{request.template_name}</TableCell>
                <TableCell>
                  <Badge variant={statusVariant(request.status)}>
                    {request.status}
                  </Badge>
                </TableCell>
                <TableCell>
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
                <TableCell>
                  {request.requested_duration_seconds / 3600}h
                </TableCell>
                <TableCell>{formatDate(request.access_expires_at)}</TableCell>
                <TableCell className="text-right">
                  {canCancel(request.status) ? (
                    <Button
                      size="sm"
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

      <Dialog
        open={createOpen}
        onOpenChange={(open) => {
          setCreateOpen(open)
          if (!open) resetForm()
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Request table access</DialogTitle>
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
          items={TABLE_PRIVILEGES.map((item) => ({ value: item, label: item }))}
          value={privilege}
          onValueChange={(value) => value && setPrivilege(value)}
        >
          <SelectTrigger id="request-privilege" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {TABLE_PRIVILEGES.map((item) => (
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
