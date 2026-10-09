"use client"

import { useCallback, useState, useTransition } from "react"
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
import { Separator } from "@/components/ui/separator"
import {
  type Account,
  createTransfer,
  type Customer,
  type DataRights,
  evaluateEligibility,
  type EligibilityResult,
  getAccount,
  getTransfer,
  loadCustomerDashboard,
  type Product,
  type Transfer,
} from "./actions"

type DashboardState = {
  customer?: Customer
  dataRights?: DataRights
  accounts?: Account[]
  products?: Product[]
  errors: string[]
}

const DEFAULT_CUSTOMER = "CUST-1001"

const CUSTOMER_OPTIONS = [
  { value: "CUST-1001", label: "CUST-1001 — Nguyễn Thị Lan" },
  { value: "CUST-1002", label: "CUST-1002 — Trần Văn Minh" },
]

const ACCOUNT_OPTIONS = [
  { value: "ACC-2001", label: "ACC-2001 — Lan · thanh toán" },
  { value: "ACC-2002", label: "ACC-2002 — Lan · tiết kiệm" },
  { value: "ACC-2003", label: "ACC-2003 — Minh · thanh toán" },
]

const CAMPAIGN_OPTIONS = [
  { value: "PREMIUM-Q4", label: "PREMIUM-Q4 — Premium Q4 upsell" },
  { value: "CASHBACK-WELCOME", label: "CASHBACK-WELCOME — New customer cashback" },
  { value: "MORTGAGE-REFI", label: "MORTGAGE-REFI — Mortgage refinance" },
  { value: "CC-STEPUP", label: "CC-STEPUP — Credit card step-up" },
]

const RISK_OPTIONS = [
  { value: "__any__" as const, label: "Any" },
  { value: "low" as const, label: "Low" },
  { value: "medium" as const, label: "Medium" },
  { value: "high" as const, label: "High" },
]

const TRANSFER_TYPE_OPTIONS = [
  { value: "transfer", label: "Transfer (debit + credit)" },
  { value: "cashback", label: "Cashback" },
  { value: "refund", label: "Refund" },
  { value: "payout", label: "Payout" },
]

const FROM_ACCOUNT_NONE = "__none__"
const ANY = "__any__"

const SEGMENT_OPTIONS = [
  { value: ANY, label: "Any" },
  { value: "retail", label: "Retail" },
  { value: "premium", label: "Premium" },
  { value: "private", label: "Private" },
  { value: "business", label: "Business" },
]

const KYC_OPTIONS = [
  { value: ANY, label: "Any" },
  { value: "verified", label: "Verified" },
  { value: "pending", label: "Pending" },
  { value: "rejected", label: "Rejected" },
]

const PRODUCT_CATEGORY_OPTIONS = [
  { value: ANY, label: "Any" },
  { value: "credit_card", label: "Credit card" },
  { value: "mortgage", label: "Mortgage" },
  { value: "loan", label: "Loan" },
  { value: "savings", label: "Savings" },
  { value: "investment", label: "Investment" },
  { value: "insurance", label: "Insurance" },
]

function formatMoney(amount: number, currency: string) {
  try {
    return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(amount)
  } catch {
    return `${amount.toFixed(2)} ${currency}`
  }
}

export function BankingView() {
  const [customerId, setCustomerId] = useState(DEFAULT_CUSTOMER)
  const [dashboard, setDashboard] = useState<DashboardState | null>(null)
  const [isLoading, startLoading] = useTransition()

  const [accountLookupId, setAccountLookupId] = useState("ACC-2001")
  const [accountDetail, setAccountDetail] = useState<Account | null>(null)
  const [accountError, setAccountError] = useState<string | null>(null)
  const [isAccountLoading, startAccountLoading] = useTransition()

  const [campaignId, setCampaignId] = useState("PREMIUM-Q4")
  const [minBalance, setMinBalance] = useState("10000")
  const [requiredSegment, setRequiredSegment] = useState("premium")
  const [requiredKyc, setRequiredKyc] = useState("verified")
  const [maxRisk, setMaxRisk] = useState<"low" | "medium" | "high" | "">("medium")
  const [requiredProductCat, setRequiredProductCat] = useState("")
  const [excludedProductCat, setExcludedProductCat] = useState("")
  const [eligibility, setEligibility] = useState<EligibilityResult | null>(null)
  const [eligibilityError, setEligibilityError] = useState<string | null>(null)
  const [isEvaluating, startEvaluating] = useTransition()

  const [transferFrom, setTransferFrom] = useState("")
  const [transferTo, setTransferTo] = useState("ACC-2003")
  const [transferAmount, setTransferAmount] = useState("5.00")
  const [transferType, setTransferType] = useState<"transfer" | "cashback" | "refund" | "payout">("cashback")
  const [transferReference, setTransferReference] = useState("CB-WELCOME")
  const [transferDescription, setTransferDescription] = useState("Welcome cashback")
  const [transfer, setTransfer] = useState<Transfer | null>(null)
  const [transferError, setTransferError] = useState<string | null>(null)
  const [isTransferring, startTransferring] = useTransition()

  const [transferLookupId, setTransferLookupId] = useState("")
  const [transferLookup, setTransferLookup] = useState<Transfer | null>(null)
  const [transferLookupError, setTransferLookupError] = useState<string | null>(null)
  const [isLookingUp, startLookingUp] = useTransition()

  const loadDashboard = useCallback(() => {
    startLoading(async () => {
      const result = await loadCustomerDashboard(customerId)
      const errors: string[] = []
      const state: DashboardState = { errors }
      if (result.customer.ok) state.customer = result.customer.data
      else errors.push(`customer: ${result.customer.error}`)
      if (result.dataRights.ok) state.dataRights = result.dataRights.data
      else errors.push(`data-rights: ${result.dataRights.error}`)
      if (result.accounts.ok) state.accounts = result.accounts.data.accounts
      else errors.push(`accounts: ${result.accounts.error}`)
      if (result.products.ok) state.products = result.products.data.products
      else errors.push(`products: ${result.products.error}`)
      setDashboard(state)
    })
  }, [customerId])

  const lookupAccount = useCallback(() => {
    startAccountLoading(async () => {
      const res = await getAccount(accountLookupId)
      if (res.ok) {
        setAccountDetail(res.data)
        setAccountError(null)
      } else {
        setAccountDetail(null)
        setAccountError(res.error)
      }
    })
  }, [accountLookupId])

  const runEligibility = useCallback(() => {
    startEvaluating(async () => {
      const rules = {
        ...(minBalance ? { minBalance: Number(minBalance) } : {}),
        ...(requiredSegment ? { requiredSegment } : {}),
        ...(requiredKyc ? { requiredKycStatus: requiredKyc } : {}),
        ...(maxRisk ? { maxRiskRating: maxRisk } : {}),
        ...(requiredProductCat ? { requiredProductCategory: requiredProductCat } : {}),
        ...(excludedProductCat ? { excludedProductCategory: excludedProductCat } : {}),
      }
      const res = await evaluateEligibility({ customerId, campaignId, rules })
      if (res.ok) {
        setEligibility(res.data)
        setEligibilityError(null)
      } else {
        setEligibility(null)
        setEligibilityError(res.error)
      }
    })
  }, [customerId, campaignId, minBalance, requiredSegment, requiredKyc, maxRisk, requiredProductCat, excludedProductCat])

  const submitTransfer = useCallback(() => {
    startTransferring(async () => {
      const amount = Number(transferAmount)
      if (!Number.isFinite(amount) || amount <= 0) {
        setTransfer(null)
        setTransferError("amount must be a positive number")
        return
      }
      const res = await createTransfer({
        fromAccountId: transferFrom || undefined,
        toAccountId: transferTo,
        amount,
        type: transferType,
        reference: transferReference || undefined,
        description: transferDescription || undefined,
      })
      if (res.ok) {
        setTransfer(res.data)
        setTransferError(null)
      } else {
        setTransfer(null)
        setTransferError(res.error)
      }
    })
  }, [transferFrom, transferTo, transferAmount, transferType, transferReference, transferDescription])

  const lookupTransfer = useCallback(() => {
    startLookingUp(async () => {
      const res = await getTransfer(transferLookupId)
      if (res.ok) {
        setTransferLookup(res.data)
        setTransferLookupError(null)
      } else {
        setTransferLookup(null)
        setTransferLookupError(res.error)
      }
    })
  }, [transferLookupId])

  const customer = dashboard?.customer
  const accounts = dashboard?.accounts ?? []
  const products = dashboard?.products ?? []
  const dataRights = dashboard?.dataRights

  return (
    <main className="space-y-6 p-6">
      <div>
        <h1 className="text-xl font-semibold">Temenos T24 — Banking Console</h1>
        <p className="text-muted-foreground text-sm">
          Backed by the Temenos simulator at <code>{"TEMENOS_URL"}</code> (default{" "}
          <code>http://localhost:4010</code>).
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Customer 360</CardTitle>
          <CardDescription>Load profile, data rights, accounts, and enrolled products.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-end gap-2">
            <div className="flex-1 space-y-1">
              <label className="text-xs font-medium" htmlFor="customerId">
                Customer ID
              </label>
              <Select
                items={CUSTOMER_OPTIONS}
                onValueChange={(v) => v && setCustomerId(v)}
                value={customerId}
              >
                <SelectTrigger className="h-9 w-full" id="customerId">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {CUSTOMER_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <Button disabled={isLoading || !customerId} onClick={loadDashboard} type="button">
              {isLoading ? "Loading…" : "Load customer"}
            </Button>
          </div>

          {dashboard?.errors.length ? (
            <ul className="rounded border border-red-200 bg-red-50 p-2 text-xs text-red-700">
              {dashboard.errors.map((e) => (
                <li key={e}>{e}</li>
              ))}
            </ul>
          ) : null}

          {customer ? (
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-1 text-sm">
                <div className="font-medium">
                  {customer.firstName} {customer.lastName}
                </div>
                <div className="text-muted-foreground">{customer.email}</div>
                <div className="text-muted-foreground">{customer.phone}</div>
                <div className="text-muted-foreground text-xs">
                  {customer.city}, {customer.country} · DOB {customer.dateOfBirth}
                </div>
                <div className="flex flex-wrap gap-1 pt-2">
                  {customer.segment ? <Badge variant="secondary">segment: {customer.segment}</Badge> : null}
                  {customer.kycStatus ? <Badge variant="secondary">kyc: {customer.kycStatus}</Badge> : null}
                  {customer.riskRating ? <Badge variant="secondary">risk: {customer.riskRating}</Badge> : null}
                </div>
              </div>
              {dataRights ? (
                <div className="space-y-1 text-sm">
                  <div className="font-medium">Data rights</div>
                  <div className="grid grid-cols-2 gap-1 text-xs">
                    <span>Access</span>
                    <span>{dataRights.canAccessData ? "yes" : "no"}</span>
                    <span>Export</span>
                    <span>{dataRights.canExportData ? "yes" : "no"}</span>
                    <span>Delete</span>
                    <span>{dataRights.canDeleteData ? "yes" : "no"}</span>
                    <span>Marketing opt-in</span>
                    <span>{dataRights.marketingOptIn ? "yes" : "no"}</span>
                    <span>Third-party sharing</span>
                    <span>{dataRights.thirdPartySharingOptIn ? "yes" : "no"}</span>
                    <span>Consent version</span>
                    <span>{dataRights.consentVersion}</span>
                  </div>
                </div>
              ) : null}
            </div>
          ) : null}

          {accounts.length > 0 ? (
            <>
              <Separator />
              <div className="space-y-2">
                <div className="text-sm font-medium">Accounts ({accounts.length})</div>
                <div className="grid gap-2 md:grid-cols-2">
                  {accounts.map((a) => (
                    <div className="rounded border p-3 text-sm" key={a.id}>
                      <div className="flex items-center justify-between">
                        <div className="font-mono text-xs">{a.id}</div>
                        <Badge variant="outline">{a.status}</Badge>
                      </div>
                      <div className="text-muted-foreground text-xs">
                        {a.type} · {a.accountNumber}
                      </div>
                      <div className="pt-1 font-medium">{formatMoney(a.balance, a.currency)}</div>
                      <div className="text-muted-foreground text-xs">
                        avail: {formatMoney(a.availableBalance, a.currency)}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </>
          ) : null}

          {products.length > 0 ? (
            <>
              <Separator />
              <div className="space-y-2">
                <div className="text-sm font-medium">Products ({products.length})</div>
                <ul className="space-y-1 text-sm">
                  {products.map((p) => (
                    <li className="flex items-center justify-between rounded border px-3 py-2" key={p.id}>
                      <div>
                        <div className="font-medium">{p.productName}</div>
                        <div className="text-muted-foreground text-xs">
                          {p.productCode} · {p.category}
                        </div>
                      </div>
                      <Badge variant="outline">{p.status}</Badge>
                    </li>
                  ))}
                </ul>
              </div>
            </>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Account lookup</CardTitle>
          <CardDescription>Fetch a single account by id.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-end gap-2">
            <div className="flex-1 space-y-1">
              <label className="text-xs font-medium" htmlFor="accountId">
                Account ID
              </label>
              <Select
                items={ACCOUNT_OPTIONS}
                onValueChange={(v) => v && setAccountLookupId(v)}
                value={accountLookupId}
              >
                <SelectTrigger className="h-9 w-full" id="accountId">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {ACCOUNT_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <Button disabled={isAccountLoading || !accountLookupId} onClick={lookupAccount} type="button">
              {isAccountLoading ? "Loading…" : "Fetch"}
            </Button>
          </div>
          {accountError ? <div className="text-xs text-red-600">{accountError}</div> : null}
          {accountDetail ? (
            <pre className="overflow-auto rounded border bg-muted/40 p-3 text-xs">
              {JSON.stringify(accountDetail, null, 2)}
            </pre>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Campaign eligibility</CardTitle>
          <CardDescription>Evaluate rules against the loaded customer.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="campaignId">Campaign ID</label>
              <Select
                items={CAMPAIGN_OPTIONS}
                onValueChange={(v) => v && setCampaignId(v)}
                value={campaignId}
              >
                <SelectTrigger className="h-9 w-full" id="campaignId">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {CAMPAIGN_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="minBalance">Min balance</label>
              <Input id="minBalance" onChange={(e) => setMinBalance(e.target.value)} value={minBalance} />
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="segment">Required segment</label>
              <Select
                items={SEGMENT_OPTIONS}
                onValueChange={(v) => {
                  if (!v) return
                  setRequiredSegment(v === ANY ? "" : v)
                }}
                value={requiredSegment === "" ? ANY : requiredSegment}
              >
                <SelectTrigger className="h-9 w-full" id="segment">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {SEGMENT_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="kyc">Required KYC</label>
              <Select
                items={KYC_OPTIONS}
                onValueChange={(v) => {
                  if (!v) return
                  setRequiredKyc(v === ANY ? "" : v)
                }}
                value={requiredKyc === "" ? ANY : requiredKyc}
              >
                <SelectTrigger className="h-9 w-full" id="kyc">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {KYC_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="maxRisk">Max risk</label>
              <Select
                items={RISK_OPTIONS}
                onValueChange={(v) => {
                  if (!v) return
                  setMaxRisk(v === "__any__" ? "" : (v as "low" | "medium" | "high"))
                }}
                value={maxRisk === "" ? "__any__" : maxRisk}
              >
                <SelectTrigger className="h-9 w-full" id="maxRisk">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {RISK_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="reqCat">Required product category</label>
              <Select
                items={PRODUCT_CATEGORY_OPTIONS}
                onValueChange={(v) => {
                  if (!v) return
                  setRequiredProductCat(v === ANY ? "" : v)
                }}
                value={requiredProductCat === "" ? ANY : requiredProductCat}
              >
                <SelectTrigger className="h-9 w-full" id="reqCat">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {PRODUCT_CATEGORY_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="excCat">Excluded product category</label>
              <Select
                items={PRODUCT_CATEGORY_OPTIONS}
                onValueChange={(v) => {
                  if (!v) return
                  setExcludedProductCat(v === ANY ? "" : v)
                }}
                value={excludedProductCat === "" ? ANY : excludedProductCat}
              >
                <SelectTrigger className="h-9 w-full" id="excCat">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {PRODUCT_CATEGORY_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
          </div>
          <Button disabled={isEvaluating || !customerId || !campaignId} onClick={runEligibility} type="button">
            {isEvaluating ? "Evaluating…" : "Evaluate"}
          </Button>
          {eligibilityError ? <div className="text-xs text-red-600">{eligibilityError}</div> : null}
          {eligibility ? (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Badge variant={eligibility.eligible ? "default" : "destructive"}>
                  {eligibility.eligible ? "ELIGIBLE" : "NOT ELIGIBLE"}
                </Badge>
                <span className="text-muted-foreground text-xs">{eligibility.evaluatedAt}</span>
              </div>
              {eligibility.reasons.length ? (
                <ul className="list-disc space-y-1 pl-5 text-xs text-red-700">
                  {eligibility.reasons.map((r) => (
                    <li key={r}>{r}</li>
                  ))}
                </ul>
              ) : null}
              <pre className="overflow-auto rounded border bg-muted/40 p-3 text-xs">
                {JSON.stringify(eligibility.context, null, 2)}
              </pre>
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Cashback / transfer</CardTitle>
          <CardDescription>
            Create a transfer. Leave &quot;from&quot; empty for cashback/refund/payout.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="fromAcc">From account (optional)</label>
              <Select
                items={[{ value: FROM_ACCOUNT_NONE, label: "(none — cashback/refund/payout)" }, ...ACCOUNT_OPTIONS]}
                onValueChange={(v) => {
                  if (!v) return
                  setTransferFrom(v === FROM_ACCOUNT_NONE ? "" : v)
                }}
                value={transferFrom === "" ? FROM_ACCOUNT_NONE : transferFrom}
              >
                <SelectTrigger className="h-9 w-full" id="fromAcc">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value={FROM_ACCOUNT_NONE}>(none — cashback/refund/payout)</SelectItem>
                    {ACCOUNT_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="toAcc">To account</label>
              <Select
                items={ACCOUNT_OPTIONS}
                onValueChange={(v) => v && setTransferTo(v)}
                value={transferTo}
              >
                <SelectTrigger className="h-9 w-full" id="toAcc">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {ACCOUNT_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="amount">Amount</label>
              <Input id="amount" onChange={(e) => setTransferAmount(e.target.value)} value={transferAmount} />
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="type">Type</label>
              <Select
                items={TRANSFER_TYPE_OPTIONS}
                onValueChange={(v) =>
                  v && setTransferType(v as "transfer" | "cashback" | "refund" | "payout")
                }
                value={transferType}
              >
                <SelectTrigger className="h-9 w-full" id="type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {TRANSFER_TYPE_OPTIONS.map((o) => (
                      <SelectItem key={o.value} value={o.value}>
                        {o.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="ref">Reference</label>
              <Input id="ref" onChange={(e) => setTransferReference(e.target.value)} value={transferReference} />
            </div>
            <div className="space-y-1">
              <label className="text-xs font-medium" htmlFor="desc">Description</label>
              <Input id="desc" onChange={(e) => setTransferDescription(e.target.value)} value={transferDescription} />
            </div>
          </div>
          <Button disabled={isTransferring || !transferTo || !transferAmount} onClick={submitTransfer} type="button">
            {isTransferring ? "Submitting…" : "Submit transfer"}
          </Button>
          {transferError ? <div className="text-xs text-red-600">{transferError}</div> : null}
          {transfer ? (
            <pre className="overflow-auto rounded border bg-muted/40 p-3 text-xs">
              {JSON.stringify(transfer, null, 2)}
            </pre>
          ) : null}

          <Separator />

          <div className="flex items-end gap-2">
            <div className="flex-1 space-y-1">
              <label className="text-xs font-medium" htmlFor="trxId">Transfer ID</label>
              <Input
                id="trxId"
                onChange={(e) => setTransferLookupId(e.target.value)}
                placeholder="TRX-…"
                value={transferLookupId}
              />
            </div>
            <Button disabled={isLookingUp || !transferLookupId} onClick={lookupTransfer} type="button">
              {isLookingUp ? "Loading…" : "Look up"}
            </Button>
          </div>
          {transferLookupError ? <div className="text-xs text-red-600">{transferLookupError}</div> : null}
          {transferLookup ? (
            <pre className="overflow-auto rounded border bg-muted/40 p-3 text-xs">
              {JSON.stringify(transferLookup, null, 2)}
            </pre>
          ) : null}
        </CardContent>
      </Card>
    </main>
  )
}
