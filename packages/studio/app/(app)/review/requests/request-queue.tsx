"use client"

import {
  IconAlertTriangle,
  IconCheck,
  IconChevronDown,
  IconClock,
  IconDatabase,
  IconLoader2,
  IconShieldCheck,
  IconUser,
  IconUsersGroup,
  IconX,
} from "@tabler/icons-react"
import { useRouter } from "next/navigation"
import { useEffect, useMemo, useState, useTransition } from "react"
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
import {
  getAccessRequestChronicleEvidence,
  reviewAccessRequest,
} from "../actions"
import type { AccessRequest, ChronicleAccessRequestEvidence } from "../types"

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

function isAutoApproved(request: AccessRequest) {
  return request.decided_by === "system:auto-approve"
}

function availableReviewTeams(request: AccessRequest, approvalTeams: string[]) {
  const assignedTeams = request.approval_teams
    .map((team) => team.team)
    .filter((team) => approvalTeams.includes(team))

  if (
    isAutoApproved(request) &&
    approvalTeams.includes(request.routed_for_team)
  ) {
    assignedTeams.push(request.routed_for_team)
  }

  return [...new Set(assignedTeams)]
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

function chronicleActionForEvent(eventType: string) {
  return `access_requests.${eventType.toLowerCase()}`
}

function isValidEvidence(evidence: ChronicleAccessRequestEvidence) {
  return (
    evidence.status === "ok" &&
    evidence.verified &&
    evidence.checkpoint_verified
  )
}

function matchChronicleEvidence(
  request: AccessRequest | null,
  evidence: ChronicleAccessRequestEvidence[],
) {
  if (!request) return []

  const used = new Set<number>()
  return request.events.map((event) => {
    const action = chronicleActionForEvent(event.event_type)
    if (!action) return null

    let bestIndex = -1
    let bestDistance = Number.POSITIVE_INFINITY
    evidence.forEach((candidate, index) => {
      if (
        used.has(index) ||
        candidate.action !== action ||
        candidate.principal !== event.actor
      ) {
        return
      }
      const distance = Math.abs(
        new Date(candidate.finished_at).getTime() -
          new Date(event.created_at).getTime(),
      )
      if (distance < bestDistance) {
        bestIndex = index
        bestDistance = distance
      }
    })

    if (bestIndex === -1) return null
    used.add(bestIndex)
    return evidence[bestIndex]
  })
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
  const [chronicleEvidence, setChronicleEvidence] = useState<
    ChronicleAccessRequestEvidence[]
  >([])
  const [chronicleStatus, setChronicleStatus] = useState<
    "idle" | "loading" | "loaded" | "error"
  >("idle")
  const [chronicleError, setChronicleError] = useState<string | null>(null)
  const [expandedProofIndex, setExpandedProofIndex] = useState<number | null>(
    null,
  )
  const [isReviewing, startReview] = useTransition()

  const requestTeams = useMemo(() => {
    if (!selected) return []
    return availableReviewTeams(selected, approvalTeams)
  }, [approvalTeams, selected])

  const evidenceByEvent = useMemo(
    () => matchChronicleEvidence(selected, chronicleEvidence),
    [chronicleEvidence, selected],
  )

  useEffect(() => {
    if (!selected) {
      setChronicleEvidence([])
      setChronicleStatus("idle")
      setChronicleError(null)
      setExpandedProofIndex(null)
      return
    }

    let active = true
    setChronicleEvidence([])
    setChronicleStatus("loading")
    setChronicleError(null)
    setExpandedProofIndex(null)
    getAccessRequestChronicleEvidence(selected.id).then((result) => {
      if (!active) return
      if (result.ok) {
        setChronicleEvidence(result.data)
        setChronicleStatus("loaded")
      } else {
        setChronicleStatus("error")
        setChronicleError(result.error)
      }
    })

    return () => {
      active = false
    }
  }, [selected])

  function openRequest(request: AccessRequest) {
    const availableTeams = availableReviewTeams(request, approvalTeams)
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
          decision === "reject"
            ? "Request rejected."
            : result.data.status === "APPROVED"
              ? "Request approved."
              : "Your approval was recorded.",
      })
      router.refresh()
      setSelected(null)
    })
  }

  const canApprove = selected?.status === "AWAITING_APPROVAL"
  const canReject =
    selected?.status === "AWAITING_APPROVAL" || selected?.status === "APPROVED"
  const canAct = (canApprove || canReject) && requestTeams.length > 0

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
              <TableHead>Requested by</TableHead>
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
                  colSpan={8}
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
                    <div className="flex flex-col items-start gap-1">
                      <Badge variant={statusVariant(request.status)}>
                        {request.status}
                      </Badge>
                      {isAutoApproved(request) ? (
                        <Badge variant="outline">Auto-approved</Badge>
                      ) : null}
                    </div>
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
                      {request.requested_by}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <IconUsersGroup className="size-4 text-muted-foreground" />
                      {request.routed_for_team}
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
                      {request.status === "AWAITING_APPROVAL" ||
                      request.status === "APPROVED"
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
                  {isAutoApproved(selected) ? (
                    <Badge variant="outline">Auto-approved</Badge>
                  ) : null}
                  <span className="font-mono text-xs text-muted-foreground">
                    {shortId(selected.id)}
                  </span>
                </div>
                <DialogTitle className="text-base">
                  {selected.target_full_name}
                </DialogTitle>
                <DialogDescription>
                  Requested by {selected.requested_by} for{" "}
                  {selected.routed_for_team}
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
                      {isAutoApproved(selected)
                        ? "Automatic"
                        : selected.approval_strategy}
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

                {selected.contract ? (
                  <div>
                    <h3 className="mb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                      Data contract decision
                    </h3>
                    <div className="space-y-3 rounded-md border bg-card p-3 text-sm">
                      <div className="flex flex-wrap items-center gap-2">
                        <Badge variant="outline">
                          Version {selected.contract.version}
                        </Badge>
                        <Badge
                          variant={
                            selected.contract.evaluation_status === "PASSED"
                              ? "outline"
                              : "secondary"
                          }
                        >
                          {selected.contract.evaluation_status}
                        </Badge>
                        <span className="text-muted-foreground">
                          {selected.purpose_code?.replaceAll("_", " ")} ·{" "}
                          {selected.consumer_context.consumer_domain}
                        </span>
                      </div>
                      <div className="grid gap-2">
                        {selected.contract.evaluations.map((evaluation) => (
                          <div
                            key={evaluation.rule_key}
                            className="flex items-start justify-between gap-3 rounded bg-muted/40 px-2 py-1.5"
                          >
                            <div>
                              <div className="font-medium">
                                {evaluation.name}
                              </div>
                              <div className="text-xs text-muted-foreground">
                                {evaluation.message}
                              </div>
                            </div>
                            <Badge
                              variant={
                                evaluation.status === "PASS"
                                  ? "outline"
                                  : "secondary"
                              }
                            >
                              {evaluation.status}
                            </Badge>
                          </div>
                        ))}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Processing:{" "}
                        {selected.consumer_context.processing_countries?.join(
                          ", ",
                        ) ?? "—"}
                        {selected.consumer_context.cross_border_transfer
                          ? " · cross-border"
                          : ""}
                        {selected.consumer_context.ai_use
                          ? " · AI/model use"
                          : ""}
                      </div>
                    </div>
                  </div>
                ) : null}

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

                <div>
                  <div className="mb-2 flex items-center justify-between gap-3">
                    <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                      Events
                    </h3>
                    {chronicleStatus === "loading" ? (
                      <span className="flex items-center gap-1 text-xs text-muted-foreground">
                        <IconLoader2 className="size-3.5 animate-spin" />
                        Checking Chronicle
                      </span>
                    ) : null}
                  </div>
                  <div className="space-y-2">
                    {selected.events.map((event, index) => {
                      const evidence = evidenceByEvent[index]
                      const validProof = evidence
                        ? isValidEvidence(evidence)
                        : false
                      return (
                        <div
                          key={`${event.event_type}:${event.actor}:${event.created_at}`}
                          className="rounded-md border bg-card p-3 text-sm"
                        >
                          <div className="flex items-start justify-between gap-3">
                            <div className="flex min-w-0 flex-col items-start gap-1.5">
                              <Badge variant={statusVariant(event.event_type)}>
                                {event.event_type}
                              </Badge>
                              <span className="break-all font-medium">
                                {event.actor}
                              </span>
                            </div>
                            <div className="flex shrink-0 justify-end">
                              {validProof ? (
                                <button
                                  type="button"
                                  aria-controls={`chronicle-proof-${evidence?.index}`}
                                  aria-expanded={
                                    expandedProofIndex === evidence?.index
                                  }
                                  className="rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                                  onClick={() =>
                                    setExpandedProofIndex((current) =>
                                      current === evidence?.index
                                        ? null
                                        : (evidence?.index ?? null),
                                    )
                                  }
                                >
                                  <Badge className="cursor-pointer bg-emerald-600 text-white hover:bg-emerald-700">
                                    <IconShieldCheck />
                                    Valid inclusion proof
                                    <IconChevronDown
                                      className={
                                        expandedProofIndex === evidence?.index
                                          ? "rotate-180 transition-transform"
                                          : "transition-transform"
                                      }
                                    />
                                  </Badge>
                                </button>
                              ) : null}
                              {evidence && !validProof ? (
                                <Badge variant="destructive">
                                  Chronicle verification failed
                                </Badge>
                              ) : null}
                              {chronicleStatus === "loaded" && !evidence ? (
                                <Badge variant="outline">
                                  No Chronicle proof
                                </Badge>
                              ) : null}
                            </div>
                          </div>
                          {validProof &&
                          evidence &&
                          expandedProofIndex === evidence.index ? (
                            <div
                              id={`chronicle-proof-${evidence.index}`}
                              className="mt-3 space-y-3 rounded-md border border-emerald-200 bg-emerald-50/50 p-3 dark:border-emerald-900 dark:bg-emerald-950/20"
                            >
                              <div className="flex flex-wrap items-center justify-between gap-2">
                                <div>
                                  <div className="font-medium">
                                    Cryptographic verification
                                  </div>
                                  <div className="text-xs text-muted-foreground">
                                    {`Chronicle entry #${evidence.index} · verified ${formatDate(evidence.finished_at)}`}
                                  </div>
                                </div>
                                <Badge variant="outline">
                                  {evidence.proof_node_count} proof nodes
                                </Badge>
                              </div>
                              <div className="grid gap-2 text-xs sm:grid-cols-2">
                                <div className="rounded bg-background/70 p-2">
                                  <div className="text-muted-foreground">
                                    Inclusion path
                                  </div>
                                  <div className="mt-1 flex items-center gap-1 font-medium text-emerald-700 dark:text-emerald-400">
                                    <IconCheck className="size-3.5" /> Verified
                                  </div>
                                </div>
                                <div className="rounded bg-background/70 p-2">
                                  <div className="text-muted-foreground">
                                    Checkpoint signature
                                  </div>
                                  <div className="mt-1 flex items-center gap-1 font-medium text-emerald-700 dark:text-emerald-400">
                                    <IconCheck className="size-3.5" /> Verified
                                  </div>
                                </div>
                              </div>
                              <dl className="grid gap-2 text-xs">
                                <div className="grid gap-1 sm:grid-cols-[7rem_1fr]">
                                  <dt className="text-muted-foreground">
                                    Action
                                  </dt>
                                  <dd className="font-mono">
                                    {evidence.action}
                                  </dd>
                                </div>
                                <div className="grid gap-1 sm:grid-cols-[7rem_1fr]">
                                  <dt className="text-muted-foreground">
                                    Tree size
                                  </dt>
                                  <dd className="font-mono">
                                    {evidence.tree_size.toLocaleString()}
                                  </dd>
                                </div>
                                <div className="grid gap-1 sm:grid-cols-[7rem_1fr]">
                                  <dt className="text-muted-foreground">
                                    Leaf hash
                                  </dt>
                                  <dd className="break-all font-mono">
                                    {evidence.leaf_hash}
                                  </dd>
                                </div>
                                <div className="grid gap-1 sm:grid-cols-[7rem_1fr]">
                                  <dt className="text-muted-foreground">
                                    Checkpoint root
                                  </dt>
                                  <dd className="break-all font-mono">
                                    {evidence.root_hash}
                                  </dd>
                                </div>
                              </dl>
                            </div>
                          ) : null}
                          {event.comment ? (
                            <p className="mt-2 text-muted-foreground">
                              {event.comment}
                            </p>
                          ) : null}
                          <div className="mt-2 flex items-center gap-1 text-xs text-muted-foreground">
                            <IconClock className="size-3.5" />
                            {formatDate(event.created_at)}
                            {evidence ? (
                              <span className="font-mono">
                                · Chronicle #{evidence.index}
                              </span>
                            ) : null}
                          </div>
                        </div>
                      )
                    })}
                    {selected.events.length === 0 ? (
                      <p className="rounded-md border p-3 text-sm text-muted-foreground">
                        No events recorded.
                      </p>
                    ) : null}
                  </div>
                  {chronicleStatus === "error" ? (
                    <p className="mt-2 flex items-center gap-1 text-xs text-destructive">
                      <IconAlertTriangle className="size-3.5" />
                      Chronicle verification unavailable: {chronicleError}
                    </p>
                  ) : null}
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
                    {isReviewing
                      ? "Saving…"
                      : canApprove
                        ? "Reject"
                        : "Reject access"}
                  </Button>
                  {canApprove ? (
                    <Button
                      disabled={isReviewing || !approvalTeam}
                      onClick={() => submitReview("approve")}
                    >
                      <IconCheck />
                      {isReviewing ? "Saving…" : "Approve"}
                    </Button>
                  ) : null}
                </DialogFooter>
              ) : null}
            </>
          ) : null}
        </DialogContent>
      </Dialog>
    </div>
  )
}
