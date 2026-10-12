"use client"

import {
  IconAlertTriangle,
  IconCheck,
  IconDatabase,
  IconUser,
  IconX,
} from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useMemo, useState, useTransition } from "react"
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
import { reviewAccessRequest } from "../actions"
import type { AccessRequest } from "../types"

type RequestQueueProps = {
  initialRequests: AccessRequest[]
  approvalTeams: string[]
  initialError?: string
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value))
}

function formatDuration(seconds: number) {
  if (seconds % 86400 === 0) return `${seconds / 86400}d`
  if (seconds % 3600 === 0) return `${seconds / 3600}h`
  if (seconds % 60 === 0) return `${seconds / 60}m`
  return `${seconds}s`
}

function shortId(id: string) {
  return id.slice(0, 8)
}

function statusVariant(
  status: string,
): "default" | "secondary" | "outline" | "destructive" {
  switch (status) {
    case "APPROVED":
    case "FULFILLED":
      return "default"
    case "REJECTED":
    case "FAILED":
      return "destructive"
    case "AWAITING_APPROVAL":
      return "secondary"
    default:
      return "outline"
  }
}

export function RequestQueue({
  initialRequests,
  approvalTeams,
  initialError,
}: RequestQueueProps) {
  const router = useRouter()
  const [selected, setSelected] = useState<AccessRequest | null>(null)
  const [approvalTeam, setApprovalTeam] = useState("")
  const [comment, setComment] = useState("")
  const [resultMessage, setResultMessage] = useState<{
    kind: "success" | "error"
    text: string
  } | null>(null)
  const [isReviewing, startReview] = useTransition()

  const requestTeams = useMemo(() => {
    if (!selected) return []
    return selected.approval_teams
      .map((team) => team.team)
      .filter((team) => approvalTeams.includes(team))
  }, [approvalTeams, selected])

  function openRequest(request: AccessRequest) {
    const availableTeams = request.approval_teams
      .map((team) => team.team)
      .filter((team) => approvalTeams.includes(team))
    setSelected(request)
    setApprovalTeam(availableTeams[0] ?? "")
    setComment("")
    setResultMessage(null)
  }

  function submitReview(decision: "approve" | "reject") {
    if (!selected || !approvalTeam) return
    startReview(async () => {
      const result = await reviewAccessRequest({
        requestId: selected.id,
        approvalTeam,
        decision,
        comment: comment.trim() || undefined,
      })
      if (!result.ok) {
        setResultMessage({ kind: "error", text: result.error })
        return
      }
      setResultMessage({
        kind: "success",
        text:
          result.data.status === "APPROVED"
            ? "Request approved."
            : "Your approval was recorded.",
      })
      router.refresh()
      setSelected(null)
    })
  }

  const canAct = selected?.status === "AWAITING_APPROVAL"

  if (initialError) {
    return (
      <div className="flex items-center gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-xs text-destructive">
        <IconAlertTriangle className="size-4 shrink-0" />
        {initialError}
      </div>
    )
  }

  return (
    <div className="w-full">
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="pl-4">Request</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Resource</TableHead>
              <TableHead>Requested for</TableHead>
              <TableHead>Duration</TableHead>
              <TableHead>Submitted</TableHead>
              <TableHead className="pr-4 text-right">Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialRequests.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={7}
                  className="h-24 text-center text-muted-foreground"
                >
                  No access requests.
                </TableCell>
              </TableRow>
            ) : (
              initialRequests.map((request) => (
                <TableRow key={request.id}>
                  <TableCell className="pl-4">
                    <div className="flex flex-col">
                      <span className="font-medium">
                        {request.template_name}
                      </span>
                      <span className="font-mono text-muted-foreground">
                        {shortId(request.id)}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant={statusVariant(request.status)}>
                      {request.status}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <IconDatabase className="size-4 shrink-0 text-muted-foreground" />
                      <div className="min-w-0">
                        <div className="truncate font-medium">
                          {request.target_full_name}
                        </div>
                        <div className="text-muted-foreground">
                          {request.target_securable_type}
                        </div>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <IconUser className="size-4 text-muted-foreground" />
                      {request.requested_for}
                    </div>
                  </TableCell>
                  <TableCell>
                    {formatDuration(request.requested_duration_seconds)}
                  </TableCell>
                  <TableCell>{formatDate(request.submitted_at)}</TableCell>
                  <TableCell className="pr-4 text-right">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => openRequest(request)}
                    >
                      {request.status === "AWAITING_APPROVAL"
                        ? "Review"
                        : "View"}
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog
        open={selected !== null}
        onOpenChange={(open) => {
          if (!open && !isReviewing) setSelected(null)
        }}
      >
        <DialogContent className="max-w-2xl">
          {selected ? (
            <>
              <DialogHeader>
                <div className="flex items-center gap-2">
                  <Badge variant={statusVariant(selected.status)}>
                    {selected.status}
                  </Badge>
                  <Badge variant="secondary">
                    {selected.target_securable_type}
                  </Badge>
                  <span className="font-mono text-xs text-muted-foreground">
                    {shortId(selected.id)}
                  </span>
                </div>
                <DialogTitle className="text-base">
                  {selected.target_full_name}
                </DialogTitle>
                <DialogDescription>
                  Requested by {selected.requested_by} for{" "}
                  {selected.requested_for}
                </DialogDescription>
              </DialogHeader>

              <div className="max-h-[60vh] space-y-5 overflow-y-auto py-2 pr-1">
                <div className="grid gap-3 sm:grid-cols-4">
                  <div className="rounded-md bg-muted/50 p-3">
                    <div className="text-xs text-muted-foreground">Policy</div>
                    <div className="mt-1 font-medium">
                      {selected.template_name}
                    </div>
                  </div>
                  <div className="rounded-md bg-muted/50 p-3">
                    <div className="text-xs text-muted-foreground">
                      Duration
                    </div>
                    <div className="mt-1 font-medium">
                      {formatDuration(selected.requested_duration_seconds)}
                    </div>
                  </div>
                  <div className="rounded-md bg-muted/50 p-3">
                    <div className="text-xs text-muted-foreground">
                      Strategy
                    </div>
                    <div className="mt-1 font-medium">
                      {selected.approval_strategy}
                    </div>
                  </div>
                  <div className="rounded-md bg-muted/50 p-3">
                    <div className="text-xs text-muted-foreground">Expires</div>
                    <div className="mt-1 font-medium">
                      {selected.access_expires_at
                        ? formatDate(selected.access_expires_at)
                        : "After approval"}
                    </div>
                  </div>
                </div>

                <div>
                  <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    Reason
                  </h3>
                  <p className="rounded-md border bg-card p-3 text-sm">
                    {selected.reason}
                  </p>
                </div>

                <div>
                  <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    Privileges
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    {selected.grants.map((grant) => (
                      <Badge
                        key={`${grant.full_name}:${grant.privilege}`}
                        variant="outline"
                      >
                        {grant.privilege} · {grant.full_name}
                      </Badge>
                    ))}
                  </div>
                </div>

                {canAct ? (
                  <>
                    <div className="grid gap-2">
                      <label
                        className="text-xs font-medium"
                        htmlFor="approval-team"
                      >
                        Approve as team
                      </label>
                      <Select
                        items={requestTeams.map((team) => ({
                          value: team,
                          label: team,
                        }))}
                        value={approvalTeam}
                        onValueChange={(value) =>
                          value && setApprovalTeam(value)
                        }
                      >
                        <SelectTrigger id="approval-team" className="w-full">
                          <SelectValue placeholder="Select approval team" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectGroup>
                            {requestTeams.map((team) => (
                              <SelectItem key={team} value={team}>
                                {team}
                              </SelectItem>
                            ))}
                          </SelectGroup>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="grid gap-2">
                      <label
                        className="text-xs font-medium"
                        htmlFor="review-comment"
                      >
                        Comment
                      </label>
                      <Textarea
                        id="review-comment"
                        value={comment}
                        onChange={(event) => setComment(event.target.value)}
                        placeholder="Optional"
                        rows={3}
                      />
                    </div>
                  </>
                ) : selected.reviews.length > 0 ? (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                      Reviews
                    </h3>
                    <div className="space-y-2">
                      {selected.reviews.map((review) => (
                        <div
                          key={`${review.approval_team}:${review.reviewer}:${review.created_at}`}
                          className="rounded-md border p-2 text-sm"
                        >
                          <div className="flex items-center gap-2">
                            <Badge
                              variant={
                                review.decision === "APPROVED"
                                  ? "default"
                                  : "destructive"
                              }
                            >
                              {review.decision}
                            </Badge>
                            <span className="font-medium">
                              {review.reviewer}
                            </span>
                            <span className="text-muted-foreground">
                              {review.approval_team}
                            </span>
                          </div>
                          {review.comment ? (
                            <p className="mt-1 text-muted-foreground">
                              {review.comment}
                            </p>
                          ) : null}
                        </div>
                      ))}
                    </div>
                  </div>
                ) : null}

                {resultMessage ? (
                  <p
                    className={
                      resultMessage.kind === "error"
                        ? "text-xs text-destructive"
                        : "text-xs text-green-700"
                    }
                  >
                    {resultMessage.text}
                  </p>
                ) : null}
              </div>

              {canAct ? (
                <DialogFooter>
                  <Button
                    variant="outline"
                    disabled={isReviewing || !approvalTeam}
                    onClick={() => submitReview("reject")}
                  >
                    <IconX />
                    {isReviewing ? "Saving…" : "Reject"}
                  </Button>
                  <Button
                    disabled={isReviewing || !approvalTeam}
                    onClick={() => submitReview("approve")}
                  >
                    <IconCheck />
                    {isReviewing ? "Saving…" : "Approve"}
                  </Button>
                </DialogFooter>
              ) : null}
            </>
          ) : null}
        </DialogContent>
      </Dialog>
    </div>
  )
}
