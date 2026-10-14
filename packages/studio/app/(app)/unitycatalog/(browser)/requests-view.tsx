"use client"

import { IconPlus } from "@tabler/icons-react"
import { cn } from "cn"
import { useEffect, useState, useTransition } from "react"
import {
  type Control,
  Controller,
  type FieldErrors,
  type UseFormRegister,
  useForm,
} from "react-hook-form"
import {
  cancelAccessRequest,
  createAccessRequest,
  updateAccessRequest,
} from "@/app/(app)/review/actions"
import type { AccessRequest } from "@/app/(app)/review/types"
import type {
  DataContract,
  DataQualityRun,
} from "@/app/(app)/unitycatalog/(browser)/[catalog]/[schema]/[table]/contract/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Combobox,
  ComboboxChip,
  ComboboxChips,
  ComboboxChipsInput,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxItem,
  ComboboxList,
  ComboboxValue,
  useComboboxAnchor,
} from "@/components/ui/combobox"
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
  table: ["SELECT", "MODIFY"],
} as const

type RequestSecurableType = keyof typeof PRIVILEGES_BY_TYPE

type CreateAccessRequestForm = {
  team: string
  privilege: string
  durationHours: number
  purposeCode: string
  consumerDomain: string
  processingCountries: string[]
  crossBorderTransfer: boolean
  aiUse: boolean
  intendedOutput: string
  acknowledgedTerms: boolean
  reason: string
}

export function RequestsView({
  securableType,
  fullName,
  teams,
  initialRequests,
  dataContract,
  latestQualityRun,
  initialError,
}: {
  securableType: RequestSecurableType
  fullName: string
  teams: string[]
  initialRequests: AccessRequest[]
  dataContract?: DataContract | null
  latestQualityRun?: DataQualityRun
  initialError?: string
}) {
  const privileges = PRIVILEGES_BY_TYPE[securableType]
  const contractReady =
    securableType !== "table" || dataContract?.sync_state === "IN_SYNC"
  const [requests, setRequests] = useState(initialRequests)
  const [message, setMessage] = useState<{
    kind: "error" | "success"
    text: string
  } | null>(initialError ? { kind: "error", text: initialError } : null)
  const [createOpen, setCreateOpen] = useState(false)
  const [selected, setSelected] = useState<AccessRequest | null>(null)
  const [durationHours, setDurationHours] = useState("8")
  const [reason, setReason] = useState("")
  const [isPending, startTransition] = useTransition()
  const {
    control: createControl,
    register: registerCreate,
    handleSubmit: handleCreateSubmit,
    reset: resetCreateForm,
    setError: setCreateError,
    formState: { errors: createErrors },
  } = useForm<CreateAccessRequestForm>({
    defaultValues: {
      team: teams[0] ?? "",
      privilege: privileges[0],
      durationHours: 8,
      purposeCode: "ANALYTICS",
      consumerDomain: "",
      processingCountries: ["VN"],
      crossBorderTransfer: false,
      aiUse: false,
      intendedOutput: "",
      acknowledgedTerms: false,
      reason: "",
    },
  })

  useEffect(() => setRequests(initialRequests), [initialRequests])

  function submitCreate(values: CreateAccessRequestForm) {
    startTransition(async () => {
      const result = await createAccessRequest({
        routed_for_team: values.team,
        target_securable_type: securableType,
        target_full_name: fullName,
        requested_privilege: values.privilege,
        purpose_code:
          securableType === "table" ? values.purposeCode : undefined,
        consumer_context:
          securableType === "table"
            ? {
                consumer_domain: values.consumerDomain.trim(),
                processing_countries: values.processingCountries,
                cross_border_transfer: values.crossBorderTransfer,
                ai_use: values.aiUse,
                intended_output: values.intendedOutput.trim() || undefined,
              }
            : undefined,
        acknowledged_contract_terms:
          securableType === "table" ? values.acknowledgedTerms : undefined,
        reason: values.reason.trim(),
        duration_hours: values.durationHours,
      })
      if (!result.ok) {
        setCreateError("root.server", { message: result.error })
        return
      }
      setRequests((items) => [result.data, ...items])
      setMessage({ kind: "success", text: "Access request submitted." })
      setCreateOpen(false)
      resetCreateForm()
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
      <div className="flex justify-end">
        <Button
          size="sm"
          disabled={!contractReady}
          onClick={() => setCreateOpen(true)}
        >
          <IconPlus /> Request
        </Button>
      </div>

      {securableType === "table" && !dataContract ? (
        <div className="rounded border border-amber-500/40 px-3 py-2 text-xs text-amber-700 dark:text-amber-400">
          This table needs an active data contract before access can be
          requested.
        </div>
      ) : securableType === "table" && !contractReady ? (
        <div className="rounded border border-amber-500/40 px-3 py-2 text-xs text-amber-700 dark:text-amber-400">
          The data contract is {dataContract?.sync_state}. Reconcile it with
          Unity Catalog before requesting access.
        </div>
      ) : null}

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
                    "cursor-pointer hover:bg-muted/30",
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
          if (!open) resetCreateForm()
        }}
      >
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Request {securableType} access</DialogTitle>
            <DialogDescription>
              Request temporary access to{" "}
              <span className="font-mono">{fullName}</span>.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleCreateSubmit(submitCreate)}>
            <RequestFields
              teams={teams}
              privileges={privileges}
              control={createControl}
              register={registerCreate}
              errors={createErrors}
              dataContract={dataContract}
              latestQualityRun={latestQualityRun}
            />
            {createErrors.root?.server ? (
              <div
                className="mt-4 rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive"
                role="alert"
              >
                {createErrors.root.server.message}
              </div>
            ) : null}
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                disabled={isPending}
                onClick={() => setCreateOpen(false)}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={isPending}>
                Request
              </Button>
            </DialogFooter>
          </form>
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
            {selected?.contract ? (
              <ContractBindingSummary request={selected} />
            ) : null}
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
  privileges,
  control,
  register,
  errors,
  dataContract,
  latestQualityRun,
}: {
  teams: string[]
  privileges: readonly string[]
  control: Control<CreateAccessRequestForm>
  register: UseFormRegister<CreateAccessRequestForm>
  errors: FieldErrors<CreateAccessRequestForm>
  dataContract?: DataContract | null
  latestQualityRun?: DataQualityRun
}) {
  return (
    <div className="space-y-4">
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-team">
          Team
        </label>
        <Controller
          name="team"
          control={control}
          rules={{ required: "Team is required." }}
          render={({ field }) => (
            <Select
              items={teams.map((item) => ({ value: item, label: item }))}
              value={field.value}
              onValueChange={(value) => value && field.onChange(value)}
            >
              <SelectTrigger
                id="request-team"
                className="w-full"
                aria-invalid={!!errors.team}
                aria-describedby={
                  errors.team ? "request-team-error" : undefined
                }
              >
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
          )}
        />
        {errors.team ? (
          <p
            id="request-team-error"
            className="text-xs text-destructive"
            role="alert"
          >
            {errors.team.message}
          </p>
        ) : null}
      </div>
      {dataContract ? (
        <ContractRequestFields
          contract={dataContract}
          latestQualityRun={latestQualityRun}
          control={control}
          register={register}
          errors={errors}
        />
      ) : null}
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-privilege">
          Privilege
        </label>
        <Controller
          name="privilege"
          control={control}
          rules={{ required: "Privilege is required." }}
          render={({ field }) => (
            <Select
              items={privileges.map((item) => ({ value: item, label: item }))}
              value={field.value}
              onValueChange={(value) => value && field.onChange(value)}
            >
              <SelectTrigger
                id="request-privilege"
                className="w-full"
                aria-invalid={!!errors.privilege}
                aria-describedby={
                  errors.privilege ? "request-privilege-error" : undefined
                }
              >
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
          )}
        />
        {errors.privilege ? (
          <p
            id="request-privilege-error"
            className="text-xs text-destructive"
            role="alert"
          >
            {errors.privilege.message}
          </p>
        ) : null}
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
          aria-invalid={!!errors.durationHours}
          aria-describedby={
            errors.durationHours ? "request-duration-error" : undefined
          }
          {...register("durationHours", {
            valueAsNumber: true,
            validate: (value) =>
              (Number.isInteger(value) && value > 0) ||
              "Enter a positive whole-hour duration.",
          })}
        />
        {errors.durationHours ? (
          <p
            id="request-duration-error"
            className="text-xs text-destructive"
            role="alert"
          >
            {errors.durationHours.message}
          </p>
        ) : null}
      </div>
      <div className="grid gap-2">
        <label className="text-xs font-medium" htmlFor="request-reason">
          Reason
        </label>
        <Textarea
          id="request-reason"
          rows={3}
          aria-invalid={!!errors.reason}
          aria-describedby={errors.reason ? "request-reason-error" : undefined}
          {...register("reason", {
            validate: (value) =>
              value.trim().length > 0 || "Reason is required.",
          })}
        />
        {errors.reason ? (
          <p
            id="request-reason-error"
            className="text-xs text-destructive"
            role="alert"
          >
            {errors.reason.message}
          </p>
        ) : null}
      </div>
    </div>
  )
}

function ContractRequestFields({
  contract,
  latestQualityRun,
  control,
  register,
  errors,
}: {
  contract: DataContract
  latestQualityRun?: DataQualityRun
  control: Control<CreateAccessRequestForm>
  register: UseFormRegister<CreateAccessRequestForm>
  errors: FieldErrors<CreateAccessRequestForm>
}) {
  const description = contract.document.description
  const limitations =
    description &&
    typeof description === "object" &&
    typeof description.limitations === "string"
      ? description.limitations
      : undefined
  const residency = (contract.document.customProperties ?? []).find(
    (property) => property.property === "residencyCountries",
  )?.value
  const countryCodes = ["VN", "TH"]
  const processingCountriesAnchor = useComboboxAnchor()

  return (
    <div className="space-y-5 rounded-lg border bg-card p-4 shadow-xs">
      <div className="space-y-2 border-b pb-4">
        <div className="flex flex-wrap items-center gap-2 text-xs">
          <span className="text-sm font-semibold">
            Contract {contract.version}
          </span>
          <Badge
            variant={
              contract.sync_state === "IN_SYNC" ? "outline" : "destructive"
            }
          >
            {contract.sync_state}
          </Badge>
          <Badge
            variant={
              latestQualityRun?.status === "PASSED" ? "outline" : "secondary"
            }
          >
            Quality {latestQualityRun?.status ?? "NOT RUN"}
          </Badge>
          <span className="text-muted-foreground">
            {contract.document.team?.name ?? contract.document.domain ?? "—"}
          </span>
        </div>
        {limitations ? (
          <p className="text-xs leading-relaxed text-muted-foreground">
            {limitations}
          </p>
        ) : null}
      </div>
      <div className="grid items-start gap-x-5 gap-y-5 sm:grid-cols-2">
        <div className="grid content-start gap-1.5">
          <label className="text-xs font-medium" htmlFor="request-purpose">
            Purpose
          </label>
          <Controller
            name="purposeCode"
            control={control}
            rules={{ required: "Purpose is required." }}
            render={({ field }) => (
              <Select
                items={[
                  "ANALYTICS",
                  "DATA_PRODUCT",
                  "OPERATIONS",
                  "COMPLIANCE",
                  "AI_MODEL",
                ].map((value) => ({
                  value,
                  label: value.replaceAll("_", " "),
                }))}
                value={field.value}
                onValueChange={(value) => value && field.onChange(value)}
              >
                <SelectTrigger id="request-purpose" className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {[
                      "ANALYTICS",
                      "DATA_PRODUCT",
                      "OPERATIONS",
                      "COMPLIANCE",
                      "AI_MODEL",
                    ].map((value) => (
                      <SelectItem key={value} value={value}>
                        {value.replaceAll("_", " ")}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            )}
          />
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            Select why access is needed. This becomes part of the approval and
            audit record.
          </p>
        </div>
        <div className="grid content-start gap-1.5">
          <label className="text-xs font-medium" htmlFor="consumer-domain">
            Consumer domain
          </label>
          <Input
            id="consumer-domain"
            aria-invalid={!!errors.consumerDomain}
            {...register("consumerDomain", {
              validate: (value) =>
                value.trim().length > 0 || "Consumer domain is required.",
            })}
          />
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            The business or data domain that will use the table and own the
            resulting output.
          </p>
        </div>
        <div className="grid content-start gap-1.5">
          <label className="text-xs font-medium" htmlFor="processing-countries">
            Processing countries
          </label>
          <Controller
            name="processingCountries"
            control={control}
            rules={{
              validate: (value) =>
                value.length > 0 || "Select at least one processing country.",
            }}
            render={({ field }) => (
              <Combobox
                items={countryCodes}
                multiple
                value={field.value}
                onValueChange={field.onChange}
              >
                <ComboboxChips
                  ref={processingCountriesAnchor}
                  aria-invalid={!!errors.processingCountries}
                >
                  <ComboboxValue>
                    {field.value.map((country) => (
                      <ComboboxChip key={country}>{country}</ComboboxChip>
                    ))}
                  </ComboboxValue>
                  <ComboboxChipsInput
                    id="processing-countries"
                    placeholder={field.value.length ? "" : "Select countries"}
                  />
                </ComboboxChips>
                <ComboboxContent anchor={processingCountriesAnchor}>
                  <ComboboxEmpty>No countries found.</ComboboxEmpty>
                  <ComboboxList>
                    {(country) => (
                      <ComboboxItem key={country} value={country}>
                        {country === "VN" ? "VN — Vietnam" : "TH — Thailand"}
                      </ComboboxItem>
                    )}
                  </ComboboxList>
                </ComboboxContent>
              </Combobox>
            )}
          />
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            Select every country where the data will be stored or processed.
            Contract residency: {formatValue(residency)}.
          </p>
          {errors.processingCountries ? (
            <p className="text-xs text-destructive">
              {errors.processingCountries.message}
            </p>
          ) : null}
        </div>
        <div className="grid content-start gap-1.5">
          <label className="text-xs font-medium" htmlFor="intended-output">
            Intended output
          </label>
          <Input
            id="intended-output"
            placeholder="Dashboard, governed data product…"
            {...register("intendedOutput")}
          />
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            Describe the dashboard, governed data product, analysis, or model
            that this access will produce.
          </p>
        </div>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <Controller
          name="crossBorderTransfer"
          control={control}
          render={({ field }) => (
            <label
              className="flex cursor-pointer items-start gap-3 rounded-md border p-3"
              htmlFor="cross-border-transfer"
            >
              <Checkbox
                id="cross-border-transfer"
                className="mt-0.5"
                checked={field.value}
                onCheckedChange={field.onChange}
              />
              <span>
                <span className="block text-xs font-medium">
                  Cross-border transfer
                </span>
                <span className="mt-0.5 block text-[11px] leading-relaxed text-muted-foreground">
                  Data will move outside its contracted residency boundary.
                </span>
              </span>
            </label>
          )}
        />
        <Controller
          name="aiUse"
          control={control}
          render={({ field }) => (
            <label
              className="flex cursor-pointer items-start gap-3 rounded-md border p-3"
              htmlFor="ai-model-use"
            >
              <Checkbox
                id="ai-model-use"
                className="mt-0.5"
                checked={field.value}
                onCheckedChange={field.onChange}
              />
              <span>
                <span className="block text-xs font-medium">
                  AI or model use
                </span>
                <span className="mt-0.5 block text-[11px] leading-relaxed text-muted-foreground">
                  Data will be used for model training, inference, or
                  evaluation.
                </span>
              </span>
            </label>
          )}
        />
      </div>
      <Controller
        name="acknowledgedTerms"
        control={control}
        rules={{ required: "You must acknowledge the contract terms." }}
        render={({ field }) => (
          <label
            className="flex cursor-pointer items-start gap-3 rounded-md bg-muted/40 p-3 text-xs"
            htmlFor="acknowledged-contract-terms"
          >
            <Checkbox
              id="acknowledged-contract-terms"
              className="mt-0.5"
              aria-invalid={!!errors.acknowledgedTerms}
              checked={field.value}
              onCheckedChange={field.onChange}
            />
            <span className="leading-relaxed">
              I acknowledge contract version {contract.version}, including its
              usage, residency, quality, and SLA terms.
            </span>
          </label>
        )}
      />
      {errors.acknowledgedTerms ? (
        <p className="text-xs text-destructive">
          {errors.acknowledgedTerms.message}
        </p>
      ) : null}
    </div>
  )
}

function ContractBindingSummary({ request }: { request: AccessRequest }) {
  const contract = request.contract
  if (!contract) return null
  return (
    <div className="space-y-2 rounded-md border p-3 text-xs">
      <div className="flex items-center justify-between gap-2">
        <span className="font-medium">Contract {contract.version}</span>
        <Badge
          variant={
            contract.evaluation_status === "PASSED" ? "outline" : "secondary"
          }
        >
          {contract.evaluation_status}
        </Badge>
      </div>
      <div className="text-muted-foreground">
        {request.purpose_code?.replaceAll("_", " ")} ·{" "}
        {request.consumer_context.consumer_domain}
      </div>
      {contract.evaluations.map((evaluation) => (
        <div
          key={evaluation.rule_key}
          className="flex items-start justify-between gap-3"
        >
          <span>{evaluation.message}</span>
          <Badge
            variant={evaluation.status === "PASS" ? "outline" : "secondary"}
          >
            {evaluation.status}
          </Badge>
        </div>
      ))}
    </div>
  )
}

function formatValue(value: unknown) {
  if (Array.isArray(value)) return value.join(", ")
  if (value === undefined || value === null) return "not declared"
  return String(value)
}
