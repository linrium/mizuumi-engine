"use server"

const BASE_URL = process.env.TEMENOS_URL ?? "http://localhost:4010"

export type Customer = {
  id: string
  firstName: string
  lastName: string
  email: string
  phone?: string | null
  dateOfBirth?: string | null
  nationalId?: string | null
  address?: string | null
  city?: string | null
  country?: string | null
  segment?: string | null
  kycStatus?: string | null
  riskRating?: string | null
  createdAt?: string | null
}

export type DataRights = {
  customerId: string
  canAccessData: boolean
  canExportData: boolean
  canDeleteData: boolean
  marketingOptIn: boolean
  thirdPartySharingOptIn: boolean
  consentVersion: string
  consentUpdatedAt: string
}

export type Account = {
  id: string
  customerId: string
  accountNumber: string
  type: string
  currency: string
  balance: number
  availableBalance: number
  status: string
  openedAt?: string | null
}

export type Product = {
  id: string
  customerId: string
  productCode: string
  productName: string
  category: string
  status: string
  enrolledAt?: string | null
}

export type Transfer = {
  id: string
  fromAccountId: string | null
  toAccountId: string
  amount: number
  currency: string
  type: string
  reference?: string | null
  description?: string | null
  status: string
  createdAt?: string | null
  completedAt?: string | null
}

export type EligibilityResult = {
  campaignId: string
  customerId: string
  eligible: boolean
  reasons: string[]
  evaluatedAt: string
  context: {
    segment: string | null
    kycStatus: string | null
    riskRating: string | null
    totalBalance: number
    accountCount: number
    productCount: number
  }
}

type ApiResult<T> = { ok: true; data: T } | { ok: false; status: number; error: string }

async function call<T>(path: string, init?: RequestInit): Promise<ApiResult<T>> {
  try {
    const response = await fetch(`${BASE_URL}${path}`, {
      ...init,
      headers: {
        "content-type": "application/json",
        ...(init?.headers ?? {}),
      },
      cache: "no-store",
    })
    const body: unknown = await response.json().catch(() => null)
    if (!response.ok) {
      const message =
        (body && typeof body === "object" && "error" in body && String((body as { error: unknown }).error)) ||
        `request_failed_${response.status}`
      return { ok: false, status: response.status, error: message }
    }
    return { ok: true, data: body as T }
  } catch (error) {
    return {
      ok: false,
      status: 0,
      error: error instanceof Error ? error.message : "network_error",
    }
  }
}

export async function getCustomer(customerId: string) {
  return call<Customer>(`/mock/customers/${encodeURIComponent(customerId)}`)
}

export async function getDataRights(customerId: string) {
  return call<DataRights>(`/mock/customers/${encodeURIComponent(customerId)}/data-rights`)
}

export async function getCustomerAccounts(customerId: string) {
  return call<{ customerId: string; accounts: Account[] }>(
    `/mock/customers/${encodeURIComponent(customerId)}/accounts`,
  )
}

export async function getCustomerProducts(customerId: string) {
  return call<{ customerId: string; products: Product[] }>(
    `/mock/customers/${encodeURIComponent(customerId)}/products`,
  )
}

export async function getAccount(accountId: string) {
  return call<Account>(`/mock/accounts/${encodeURIComponent(accountId)}`)
}

export type EligibilityRules = {
  minBalance?: number
  requiredSegment?: string
  requiredKycStatus?: string
  maxRiskRating?: "low" | "medium" | "high"
  requiredProductCategory?: string
  excludedProductCategory?: string
}

export async function evaluateEligibility(input: {
  customerId: string
  campaignId: string
  rules?: EligibilityRules
}) {
  return call<EligibilityResult>("/mock/eligibility/evaluate", {
    method: "POST",
    body: JSON.stringify(input),
  })
}

export async function createTransfer(input: {
  fromAccountId?: string
  toAccountId: string
  amount: number
  currency?: string
  type?: "transfer" | "cashback" | "refund" | "payout"
  reference?: string
  description?: string
}) {
  return call<Transfer>("/mock/transfers", {
    method: "POST",
    body: JSON.stringify(input),
  })
}

export async function getTransfer(transferId: string) {
  return call<Transfer>(`/mock/transfers/${encodeURIComponent(transferId)}`)
}

export async function loadCustomerDashboard(customerId: string) {
  const [customer, dataRights, accounts, products] = await Promise.all([
    getCustomer(customerId),
    getDataRights(customerId),
    getCustomerAccounts(customerId),
    getCustomerProducts(customerId),
  ])
  return { customer, dataRights, accounts, products }
}
