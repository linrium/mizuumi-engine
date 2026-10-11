"use client"

import {
  IconAlertTriangle,
  IconCheck,
  IconClock,
  IconDatabase,
  IconInbox,
  IconRoute,
  IconUser,
  IconX,
} from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useMemo, useState, useTransition } from "react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
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

  return (
    <section className="space-y-5">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h2 className="text-lg font-semibold">Request queue</h2>
          <p className="text-sm text-muted-foreground">
            Requests routed to your Keycloak approval teams.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {approvalTeams.map((team) => (
            <Badge key={team} variant="outline">
              {team}
            </Badge>
          ))}
        </div>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <Card size="sm">
          <CardHeader>
            <CardDescription>Awaiting review</CardDescription>
            <CardTitle className="flex items-center gap-2 text-xl">
              <IconInbox className="size-4 text-muted-foreground" />
              {initialRequests.length}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card size="sm">
          <CardHeader>
            <CardDescription>Your approval teams</CardDescription>
            <CardTitle className="flex items-center gap-2 text-xl">
              <IconRoute className="size-4 text-muted-foreground" />
              {approvalTeams.length}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card size="sm">
          <CardHeader>
            <CardDescription>Oldest request</CardDescription>
            <CardTitle className="flex items-center gap-2 text-sm">
              <IconClock className="size-4 text-muted-foreground" />
              {initialRequests.length > 0
                ? formatDate(
                    initialRequests[initialRequests.length - 1].submitted_at,
                  )
                : "Queue clear"}
            </CardTitle>
          </CardHeader>
        </Card>
      </div>

      {initialError ? (
        <Card className="border-destructive/40">
          <CardHeader>
            <div className="flex items-center gap-2 text-destructive">
              <IconAlertTriangle className="size-4" />
              <CardTitle>Could not load the request queue</CardTitle>
            </div>
            <CardDescription>{initialError}</CardDescription>
          </CardHeader>
        </Card>
      ) : approvalTeams.length === 0 ? (
        <Card>
          <CardHeader>
            <CardTitle>No approval-team memberships</CardTitle>
            <CardDescription>
              Your Keycloak account is not assigned to a group used by an
              approval route. Ask an administrator to add the appropriate group.
            </CardDescription>
          </CardHeader>
        </Card>
      ) : initialRequests.length === 0 ? (
        <Card>
          <CardHeader className="items-center py-10 text-center">
            <IconCheck className="size-8 text-muted-foreground" />
            <CardTitle>All caught up</CardTitle>
            <CardDescription>
              There are no access requests awaiting your teams.
            </CardDescription>
          </CardHeader>
        </Card>
      ) : (
        <Card>
          <CardHeader className="border-b">
            <CardTitle>Pending access</CardTitle>
            <CardDescription>
              Open a request to inspect its policy snapshot and record a
              decision.
            </CardDescription>
          </CardHeader>
          <CardContent className="px-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="pl-4">Request</TableHead>
                  <TableHead>Resource</TableHead>
                  <TableHead>Requested for</TableHead>
                  <TableHead>Duration</TableHead>
                  <TableHead>Submitted</TableHead>
                  <TableHead className="pr-4 text-right">Action</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {initialRequests.map((request) => (
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
                      <div className="flex max-w-64 items-center gap-2">
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
                        Review
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

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
                <div className="grid gap-3 sm:grid-cols-3">
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
                      Approval strategy
                    </div>
                    <div className="mt-1 font-medium">
                      {selected.approval_strategy}
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
                    Requested privileges
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
                    onValueChange={(value) => value && setApprovalTeam(value)}
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
                    Review comment
                  </label>
                  <Textarea
                    id="review-comment"
                    value={comment}
                    onChange={(event) => setComment(event.target.value)}
                    placeholder="Add context for your decision (optional)"
                    rows={3}
                  />
                </div>

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
            </>
          ) : null}
        </DialogContent>
      </Dialog>
    </section>
  )
}
