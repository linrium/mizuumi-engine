import type { Random, RegionGeneratorEngine } from "./generators/types.js"
import { getRegionGenerator, type RegionCode } from "./generators/index.js"
import {
  ascii,
  chance,
  decimal,
  integer,
  pick,
  poisson,
  weightedPick,
} from "./generators/utils.js"
import type { SyntheticPerson } from "./schemas/common.js"
import { SyntheticDatasetSchema, type SyntheticDataset } from "./schemas/dataset.js"
import type {
  CrmCampaignInteraction,
  CrmContact,
  CrmPollBatch,
} from "./schemas/organizations/crm.js"
import type {
  VietjetBooking,
  VietjetCustomer,
} from "./schemas/organizations/vietjet-air.js"
import type { VikkiWallet } from "./schemas/organizations/vikki-bank.js"
import type {
  T24Account,
  T24Customer,
  T24FundsTransfer,
  T24LoanArrangement,
} from "./schemas/temenos-t24.js"

export const sourceNames = ["hdbank", "vikkiBank", "vietjetAir", "crm"] as const
export type SourceName = (typeof sourceNames)[number]
export type ScenarioName = "clean" | "realistic" | "matcher-demo" | "stress"
export type SourceMode = "sampled" | "all"
export type CustomerSegment =
  | "young-digital"
  | "salaried"
  | "family-borrower"
  | "frequent-traveler"
  | "low-activity"
export type IdentityField =
  | "fullName"
  | "dateOfBirth"
  | "nationalId"
  | "phoneNumber"
  | "emailAddress"
  | "address"

export type EconomicProfile = {
  segment: CustomerSegment
  monthlyIncome: number
  monthlyExpenses: number
  liquidAssets: number
  digitalAffinity: number
  creditNeed: number
  travelPropensity: number
  currency: "VND" | "THB"
}

export type AnomalyManifestEntry = {
  truthId: string
  source: SourceName
  sourceRecordId: string
  field: IdentityField
  kind: "typo" | "stale-value" | "conflicting-value"
  canonicalValue: string
  observedValue: string
  severity: "low" | "medium"
}

export type GeneratedPersonRecords = {
  person: SyntheticPerson
  profile: EconomicProfile
  hdbank?: {
    customer: T24Customer
    account?: T24Account
    loan?: T24LoanArrangement
    fundsTransfers: T24FundsTransfer[]
  }
  vikkiBank?: {
    customer: T24Customer
    account?: T24Account
    wallet?: VikkiWallet
    fundsTransfers: T24FundsTransfer[]
  }
  vietjetAir?: {
    customer: VietjetCustomer
    bookings: VietjetBooking[]
  }
  crm?: {
    contact: CrmContact
    campaignInteractions: CrmCampaignInteraction[]
    pollBatch: CrmPollBatch
  }
}

export type GeneratedSyntheticData = {
  dataset: SyntheticDataset
  records: GeneratedPersonRecords[]
  manifest: AnomalyManifestEntry[]
  coverage: Record<SourceName, number>
}

export type GenerateOptions = {
  count: number
  seed: number
  sequenceStart?: number
  region?: RegionCode
  generatedAt?: Date
  scenario?: ScenarioName
  sourceMode?: SourceMode
  sources?: SourceName[]
  anomalyRate?: number
  maxAnomaliesPerPerson?: number
}

const scenarioAnomalyRates: Record<ScenarioName, number> = {
  clean: 0,
  realistic: 0.12,
  "matcher-demo": 0.35,
  stress: 0.6,
}

const sourceCombinations: readonly {
  value: readonly Exclude<SourceName, "crm">[]
  weight: number
}[] = [
  { value: ["hdbank"], weight: 12 },
  { value: ["vikkiBank"], weight: 12 },
  { value: ["vietjetAir"], weight: 10 },
  { value: ["hdbank", "vikkiBank"], weight: 18 },
  { value: ["hdbank", "vietjetAir"], weight: 15 },
  { value: ["vikkiBank", "vietjetAir"], weight: 13 },
  { value: ["hdbank", "vikkiBank", "vietjetAir"], weight: 20 },
]

function mulberry32(seed: number): Random {
  let state = seed >>> 0
  return () => {
    state += 0x6d2b79f5
    let value = state
    value = Math.imul(value ^ (value >>> 15), value | 1)
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61)
    return ((value ^ (value >>> 14)) >>> 0) / 4_294_967_296
  }
}

function hashSeed(seed: number, ...parts: (string | number)[]) {
  let hash = (2_166_136_261 ^ seed) >>> 0
  for (const character of parts.join("|")) {
    hash ^= character.charCodeAt(0)
    hash = Math.imul(hash, 16_777_619) >>> 0
  }
  return hash
}

function randomFor(seed: number, sequence: number, scope: string) {
  return mulberry32(hashSeed(seed, sequence, scope))
}

function isoDate(date: Date) {
  return date.toISOString().slice(0, 10)
}

function addDays(date: Date, days: number) {
  const result = new Date(date)
  result.setUTCDate(result.getUTCDate() + days)
  return result
}

function addMonths(date: Date, months: number) {
  const result = new Date(date)
  result.setUTCMonth(result.getUTCMonth() + months)
  return result
}

function money(amount: number, engine: RegionGeneratorEngine) {
  return { amount: Math.max(0, Math.round(amount)), currency: engine.currency }
}

function recordHeader(id: string, recordedAt: string) {
  return {
    id,
    status: "success" as const,
    transactionStatus: "Live" as const,
    versionNumber: "1",
    recordedAt,
  }
}

function createT24Customer(
  person: SyntheticPerson,
  id: string,
  companyId: string,
  recordedAt: string,
  engine: RegionGeneratorEngine,
): T24Customer {
  const birthYear = Number(person.dateOfBirth.slice(0, 4))
  return {
    application: "CUSTOMER",
    header: recordHeader(id, recordedAt),
    body: {
      personId: person.personId,
      customerMnemonic: ascii(`${person.givenName}${id.slice(-6)}`).toUpperCase(),
      sectorId: 1001,
      gender: person.gender,
      nationalityId: engine.countryCode,
      residenceId: engine.countryCode,
      displayNames: [{ displayName: person.fullName }],
      customerNames: [{ customerName: person.fullName }],
      language: engine.locale,
      dateOfBirth: person.dateOfBirth,
      communicationDevices: [{ phoneNumber: person.phoneNumber, email: person.emailAddress }],
      legalIdentifiers: [
        {
          legalId: person.nationalId,
          legalDocumentName: engine.legalDocumentName,
          countryOfIssue: engine.countryCode,
          issueDate: `${Math.max(birthYear + 25, 2021)}-01-15`,
          expiryDate: `${birthYear + 55}-12-31`,
        },
      ],
      addresses: [{ addressType: "HOME", ...person.address }],
      target: 1,
      customerStatus: 1,
      industryId: 1000,
      accountOfficerId: "AO-DIGITAL-01",
      customerCompany: companyId,
      amlCheck: "PASSED",
    },
  }
}

function createAccount(input: {
  id: string
  customerId: string
  arrangementId: string
  accountNumber: string
  person: SyntheticPerson
  engine: RegionGeneratorEngine
  companyId: string
  branchId: string
  productGroup: string
  productId: string
  balance: number
  openedAt: Date
  recordedAt: string
}): T24Account {
  const balance = money(input.balance, input.engine)
  return {
    application: "ACCOUNT",
    header: recordHeader(input.id, input.recordedAt),
    body: {
      customerId: input.customerId,
      arrangementId: input.arrangementId,
      accountNumber: input.accountNumber,
      accountTitle: input.person.fullName,
      shortTitle: input.person.fullName,
      productLine: "ACCOUNTS",
      productGroup: input.productGroup,
      productId: input.productId,
      categoryId: 1001,
      currency: input.engine.currency,
      companyId: input.companyId,
      branchId: input.branchId,
      openingDate: isoDate(input.openedAt),
      status: "ACTIVE",
      onlineActualBalance: balance,
      onlineClearedBalance: balance,
      workingBalance: balance,
    },
  }
}

function createTransfer(input: {
  id: string
  reference: string
  debtorAccount: string
  creditorAccount: string
  amount: number
  type: T24FundsTransfer["body"]["transactionType"]
  description: string
  bookedAt: Date
  recordedAt: string
  engine: RegionGeneratorEngine
}): T24FundsTransfer {
  return {
    application: "FUNDS.TRANSFER",
    header: recordHeader(input.id, input.recordedAt),
    body: {
      transactionReference: input.reference,
      debtorAccount: input.debtorAccount,
      creditorAccount: input.creditorAccount,
      transactionAmount: money(input.amount, input.engine),
      transactionType: input.type,
      remittanceInformation: input.description,
      bookingDate: isoDate(input.bookedAt),
      valueDate: isoDate(input.bookedAt),
      paymentStatus: "COMPLETED",
    },
  }
}

function bookingReference(seed: number, sequence: number, booking: number) {
  return Math.abs(seed * 10_000 + sequence * 10 + booking)
    .toString(36)
    .toUpperCase()
    .padStart(6, "0")
    .slice(-6)
}

function createProfile(
  engine: RegionGeneratorEngine,
  seed: number,
  sequence: number,
): EconomicProfile {
  const random = randomFor(seed, sequence, "economic-profile")
  const segment = weightedPick(random, [
    { value: "young-digital" as const, weight: 25 },
    { value: "salaried" as const, weight: 35 },
    { value: "family-borrower" as const, weight: 20 },
    { value: "frequent-traveler" as const, weight: 12 },
    { value: "low-activity" as const, weight: 8 },
  ])
  const baseIncome = engine.region === "vn" ? 18_000_000 : 32_000
  const incomeMultiplier: Record<CustomerSegment, number> = {
    "young-digital": 0.85,
    salaried: 1.1,
    "family-borrower": 1.25,
    "frequent-traveler": 2.1,
    "low-activity": 0.65,
  }
  const monthlyIncome = Math.round(
    baseIncome * incomeMultiplier[segment] * decimal(random, 0.65, 1.55),
  )
  const expenseRatio = decimal(random, 0.45, segment === "family-borrower" ? 0.82 : 0.72)
  const monthlyExpenses = Math.round(monthlyIncome * expenseRatio)
  const liquidAssets = Math.round(monthlyIncome * decimal(random, 0.6, 7.5))
  return {
    segment,
    monthlyIncome,
    monthlyExpenses,
    liquidAssets,
    digitalAffinity:
      segment === "young-digital" ? 0.9 : segment === "low-activity" ? 0.25 : decimal(random, 0.4, 0.8),
    creditNeed:
      segment === "family-borrower" ? 0.9 : segment === "low-activity" ? 0.2 : decimal(random, 0.3, 0.7),
    travelPropensity:
      segment === "frequent-traveler" ? 0.95 : segment === "low-activity" ? 0.15 : decimal(random, 0.25, 0.7),
    currency: engine.currency,
  }
}

function sourceAffinity(segment: CustomerSegment, source: Exclude<SourceName, "crm">) {
  const multipliers: Record<CustomerSegment, Record<Exclude<SourceName, "crm">, number>> = {
    "young-digital": { hdbank: 0.8, vikkiBank: 1.55, vietjetAir: 1.15 },
    salaried: { hdbank: 1.25, vikkiBank: 1.05, vietjetAir: 1 },
    "family-borrower": { hdbank: 1.6, vikkiBank: 0.85, vietjetAir: 0.75 },
    "frequent-traveler": { hdbank: 1.1, vikkiBank: 1.1, vietjetAir: 1.8 },
    "low-activity": { hdbank: 0.85, vikkiBank: 0.7, vietjetAir: 0.55 },
  }
  return multipliers[segment][source]
}

function chooseMembership(
  profile: EconomicProfile,
  seed: number,
  sequence: number,
  sourceMode: SourceMode,
) {
  if (sourceMode === "all") return new Set<SourceName>(sourceNames)
  const random = randomFor(seed, sequence, "source-coverage")
  const selected = weightedPick(
    random,
    sourceCombinations.map((combination) => ({
      value: combination.value,
      weight:
        combination.weight *
        combination.value.reduce(
          (multiplier, source) => multiplier * sourceAffinity(profile.segment, source),
          1,
        ),
    })),
  )
  const membership = new Set<SourceName>(selected)
  const crmProbability = profile.segment === "low-activity" ? 0.4 : 0.7
  if (chance(randomFor(seed, sequence, "crm-coverage"), crmProbability)) membership.add("crm")
  return membership
}

function changedDigit(value: string, random: Random, avoidLast = false) {
  const maximumIndex = value.length - (avoidLast ? 2 : 1)
  const index = integer(random, Math.max(0, value.length - 6), Math.max(0, maximumIndex))
  const current = Number(value[index])
  const replacement = String((current + integer(random, 1, 9)) % 10)
  return `${value.slice(0, index)}${replacement}${value.slice(index + 1)}`
}

function thaiCheckDigit(firstTwelve: string) {
  const sum = [...firstTwelve].reduce(
    (total, digit, index) => total + Number(digit) * (13 - index),
    0,
  )
  return String((11 - (sum % 11)) % 10)
}

function identityValue(person: SyntheticPerson, field: IdentityField) {
  return field === "address" ? person.address.addressLine1 : String(person[field])
}

function mutateIdentity(
  canonical: SyntheticPerson,
  source: SourceName,
  sourceRecordId: string,
  seed: number,
  sequence: number,
  scenario: ScenarioName,
  anomalyRate: number,
  maxAnomalies: number,
): { observed: SyntheticPerson; manifest: AnomalyManifestEntry[] } {
  const observed = structuredClone(canonical)
  if (!chance(randomFor(seed, sequence, "anomaly-gate"), anomalyRate)) {
    return { observed, manifest: [] }
  }
  const target = pick(randomFor(seed, sequence, "anomaly-source"), sourceNames)
  if (target !== source) return { observed, manifest: [] }

  const requestedCount = weightedPick(randomFor(seed, sequence, "anomaly-count"), [
    { value: 1, weight: 82 },
    { value: 2, weight: 15 },
    { value: 3, weight: 3 },
  ])
  const count = Math.min(requestedCount, maxAnomalies)
  const manifest: AnomalyManifestEntry[] = []
  const used = new Set<IdentityField>()
  const fieldRandom = randomFor(seed, sequence, `anomaly-fields-${source}`)
  const candidates = [
    { value: "phoneNumber" as const, weight: 25 },
    { value: "fullName" as const, weight: 25 },
    { value: "emailAddress" as const, weight: 20 },
    { value: "dateOfBirth" as const, weight: 12 },
    { value: "nationalId" as const, weight: 10 },
    { value: "address" as const, weight: source === "vietjetAir" ? 0 : 8 },
  ]
  while (used.size < count) used.add(weightedPick(fieldRandom, candidates))

  for (const field of used) {
    const canonicalValue = identityValue(canonical, field)
    let kind: AnomalyManifestEntry["kind"] = "typo"
    if (field === "phoneNumber") {
      observed.phoneNumber = changedDigit(observed.phoneNumber, fieldRandom)
    } else if (field === "fullName") {
      const parts = observed.fullName.split(" ")
      const withoutDiacritics = ascii(observed.fullName)
      if (parts.length > 2 && chance(fieldRandom, 0.6)) {
        observed.fullName = parts.filter((_, index) => index !== 1).join(" ")
      } else if (withoutDiacritics !== observed.fullName) {
        observed.fullName = withoutDiacritics
      } else {
        const characters = [...parts[0]!]
        characters.splice(Math.max(0, characters.length - 1), 1)
        observed.fullName = [characters.join(""), ...parts.slice(1)].join(" ")
      }
      observed.fullNameAscii = ascii(observed.fullName)
      kind = chance(fieldRandom, 0.5) ? "stale-value" : "typo"
    } else if (field === "emailAddress") {
      const [local, domain] = observed.emailAddress.split("@") as [string, string]
      observed.emailAddress = `${local}x@${domain}`
    } else if (field === "dateOfBirth") {
      const date = new Date(`${observed.dateOfBirth}T00:00:00.000Z`)
      observed.dateOfBirth = isoDate(addDays(date, chance(fieldRandom, 0.85) ? 1 : 365))
      kind = "conflicting-value"
    } else if (field === "nationalId") {
      if (observed.nationalityCode === "TH") {
        const firstTwelve = changedDigit(observed.nationalId.slice(0, 12), fieldRandom)
        observed.nationalId = `${firstTwelve}${thaiCheckDigit(firstTwelve)}`
      } else {
        observed.nationalId = changedDigit(observed.nationalId, fieldRandom)
      }
      kind = "conflicting-value"
    } else {
      observed.address.addressLine1 = observed.address.addressLine1.replace(
        /^\d+/,
        (number) => String(Number(number) + 2),
      )
      kind = "stale-value"
    }
    manifest.push({
      truthId: canonical.personId,
      source,
      sourceRecordId,
      field,
      kind,
      canonicalValue,
      observedValue: identityValue(observed, field),
      severity: scenario === "stress" && field === "nationalId" ? "medium" : "low",
    })
  }
  return { observed, manifest }
}

function boundedProbability(value: number) {
  return Math.max(0.05, Math.min(0.98, value))
}

function setAccountBalance(account: T24Account, amount: number, engine: RegionGeneratorEngine) {
  const balance = money(amount, engine)
  account.body.onlineActualBalance = balance
  account.body.onlineClearedBalance = balance
  account.body.workingBalance = balance
}

export function generateSyntheticData(options: GenerateOptions): GeneratedSyntheticData {
  if (!Number.isInteger(options.count) || options.count < 1 || options.count > 1_000) {
    throw new RangeError("count must be an integer between 1 and 1000")
  }
  if (!Number.isInteger(options.seed) || options.seed < 0 || options.seed > 2_147_483_647) {
    throw new RangeError("seed must be an integer between 0 and 2147483647")
  }
  const sequenceStart = options.sequenceStart ?? 0
  if (!Number.isInteger(sequenceStart) || sequenceStart < 0) {
    throw new RangeError("sequenceStart must be a non-negative integer")
  }
  const scenario = options.scenario ?? "realistic"
  const anomalyRate = options.anomalyRate ?? scenarioAnomalyRates[scenario]
  if (!Number.isFinite(anomalyRate) || anomalyRate < 0 || anomalyRate > 1) {
    throw new RangeError("anomalyRate must be between 0 and 1")
  }
  const maxAnomalies = options.maxAnomaliesPerPerson ?? 3
  if (!Number.isInteger(maxAnomalies) || maxAnomalies < 1 || maxAnomalies > 3) {
    throw new RangeError("maxAnomaliesPerPerson must be an integer between 1 and 3")
  }

  const region = options.region ?? "vn"
  const engine = getRegionGenerator(region)
  const generatedAtDate = options.generatedAt ?? new Date()
  const generatedAt = generatedAtDate.toISOString()
  const outputSources = new Set(options.sources ?? sourceNames)

  const people: SyntheticPerson[] = []
  const records: GeneratedPersonRecords[] = []
  const manifest: AnomalyManifestEntry[] = []
  const coverage: Record<SourceName, number> = { hdbank: 0, vikkiBank: 0, vietjetAir: 0, crm: 0 }
  const vikkiCustomers: T24Customer[] = []
  const vikkiAccounts: T24Account[] = []
  const vikkiWallets: VikkiWallet[] = []
  const vikkiTransfers: T24FundsTransfer[] = []
  const hdbankCustomers: T24Customer[] = []
  const hdbankAccounts: T24Account[] = []
  const hdbankLoans: T24LoanArrangement[] = []
  const hdbankTransfers: T24FundsTransfer[] = []
  const vietjetCustomers: VietjetCustomer[] = []
  const vietjetBookings: VietjetBooking[] = []
  const crmContacts: CrmContact[] = []
  const crmInteractions: CrmCampaignInteraction[] = []
  const crmPollBatches: CrmPollBatch[] = []

  for (let index = 0; index < options.count; index += 1) {
    const sequence = sequenceStart + index + 1
    const suffix = `${region}-${options.seed}-${String(sequence).padStart(4, "0")}`
    const canonical = engine.createPerson(
      randomFor(options.seed, sequence, "identity"),
      sequence,
      options.seed,
    )
    const profile = createProfile(engine, options.seed, sequence)
    const membership = chooseMembership(profile, options.seed, sequence, options.sourceMode ?? "sampled")
    const emitted = new Set([...membership].filter((source) => outputSources.has(source)))
    const record: GeneratedPersonRecords = { person: canonical, profile }
    people.push(canonical)

    const observed = {} as Record<SourceName, SyntheticPerson>
    const sourceRecordIds: Record<SourceName, string> = {
      hdbank: `HDB-C-${suffix}`,
      vikkiBank: `VIK-C-${suffix}`,
      vietjetAir: `VJ-C-${suffix}`,
      crm: `CRM-C-${suffix}`,
    }
    for (const source of membership) {
      const localPerson = structuredClone(canonical)
      localPerson.personId = `${source.toUpperCase()}-P-${suffix}`
      const mutation = mutateIdentity(
        localPerson,
        source,
        sourceRecordIds[source],
        options.seed,
        sequence,
        scenario,
        anomalyRate,
        maxAnomalies,
      )
      observed[source] = mutation.observed
      for (const entry of mutation.manifest) entry.truthId = canonical.personId
      if (emitted.has(source)) manifest.push(...mutation.manifest)
    }

    const vikkiCustomerId = sourceRecordIds.vikkiBank
    const vikkiAccountNumber = `9704${String(options.seed).padStart(10, "0").slice(-10)}${String(sequence).padStart(6, "0")}`
    const vikkiCompanyId = `${engine.countryCode}0010001`
    const vikkiFundsTransfers: T24FundsTransfer[] = []
    let vikkiCustomer: T24Customer | undefined
    let vikkiAccount: T24Account | undefined
    let wallet: VikkiWallet | undefined
    let vikkiAvailable = 0
    if (membership.has("vikkiBank")) {
      const person = observed.vikkiBank
      vikkiCustomer = createT24Customer(person, vikkiCustomerId, vikkiCompanyId, generatedAt, engine)
      const productRandom = randomFor(options.seed, sequence, "vikki-products")
      const hasAccount = chance(
        productRandom,
        boundedProbability(0.78 + (profile.digitalAffinity - 0.5) * 0.3),
      )
      if (hasAccount) {
        const openedAt = addDays(generatedAtDate, -integer(productRandom, 450, 1_500))
        vikkiAvailable = Math.round(
          Math.max(
            engine.financials.accountBalance[0] * 0.35,
            profile.liquidAssets * decimal(productRandom, 0.2, 0.65),
          ),
        )
        vikkiAccount = createAccount({
          id: `VIK-A-${suffix}`,
          customerId: vikkiCustomerId,
          arrangementId: `VIK-AA-${suffix}`,
          accountNumber: vikkiAccountNumber,
          person,
          engine,
          companyId: vikkiCompanyId,
          branchId: `VIK-${engine.countryCode}-DIGITAL`,
          productGroup: "CURRENT.ACCOUNTS",
          productId: "VIKKI.DIGITAL.ACCOUNT",
          balance: vikkiAvailable,
          openedAt,
          recordedAt: generatedAt,
        })
        if (chance(productRandom, boundedProbability(0.58 + profile.digitalAffinity * 0.25))) {
          wallet = {
            walletId: `VIK-W-${suffix}`,
            customerId: vikkiCustomerId,
            settlementAccountNumber: vikkiAccountNumber,
            walletType: "VIKKI_DIGITAL_WALLET",
            activatedAt: isoDate(addDays(openedAt, integer(productRandom, 0, 30))),
            status: "ACTIVE",
          }
          const transactionCount = chance(productRandom, 0.25)
            ? 0
            : Math.min(12, poisson(productRandom, 2.2))
          for (let transferIndex = 0; transferIndex < transactionCount; transferIndex += 1) {
            const maximum = Math.min(
              engine.financials.walletPayment[1],
              Math.floor(vikkiAvailable * 0.18),
              Math.floor((profile.monthlyIncome - profile.monthlyExpenses) * 0.3),
            )
            if (maximum < engine.financials.walletPayment[0]) break
            const amount = integer(productRandom, engine.financials.walletPayment[0], maximum)
            vikkiAvailable -= amount
            vikkiFundsTransfers.push(
              createTransfer({
                id: `VIK-FT-${suffix}-${transferIndex + 1}`,
                reference: `VIK-TXN-${suffix}-${transferIndex + 1}`,
                debtorAccount: vikkiAccountNumber,
                creditorAccount: `MERCHANT-${engine.countryCode}-${transferIndex + 1}`,
                amount,
                type: "WALLET_PAYMENT",
                description: engine.walletDescription(transferIndex + 1),
                bookedAt: addDays(generatedAtDate, -20 - transferIndex * 15),
                recordedAt: generatedAt,
                engine,
              }),
            )
          }
        }
      }
    }

    const hdbankCustomerId = sourceRecordIds.hdbank
    const hdbankAccountNumber = `9705${String(options.seed).padStart(10, "0").slice(-10)}${String(sequence).padStart(6, "0")}`
    const hdbankCompanyId = `${engine.countryCode}0020001`
    const hdbankFundsTransfers: T24FundsTransfer[] = []
    let hdbankCustomer: T24Customer | undefined
    let hdbankAccount: T24Account | undefined
    let loan: T24LoanArrangement | undefined
    let hdbankAvailable = 0
    if (membership.has("hdbank")) {
      const person = observed.hdbank
      hdbankCustomer = createT24Customer(person, hdbankCustomerId, hdbankCompanyId, generatedAt, engine)
      const productRandom = randomFor(options.seed, sequence, "hdbank-products")
      const hasAccount = chance(
        productRandom,
        boundedProbability(0.84 + (profile.segment === "salaried" ? 0.1 : 0)),
      )
      if (hasAccount) {
        const openedAt = addDays(generatedAtDate, -integer(productRandom, 365, 2_400))
        hdbankAvailable = Math.round(
          Math.max(
            engine.financials.accountBalance[0] * 0.4,
            profile.liquidAssets * decimal(productRandom, 0.25, 0.75),
          ),
        )
        hdbankAccount = createAccount({
          id: `HDB-A-${suffix}`,
          customerId: hdbankCustomerId,
          arrangementId: `HDB-AA-ACCOUNT-${suffix}`,
          accountNumber: hdbankAccountNumber,
          person,
          engine,
          companyId: hdbankCompanyId,
          branchId: `HDB-${engine.countryCode}-01`,
          productGroup: "CURRENT.ACCOUNTS",
          productId: "HDB.PAYMENT.ACCOUNT",
          balance: hdbankAvailable,
          openedAt,
          recordedAt: generatedAt,
        })
        const annualRate = 0.108
        const monthlyRate = annualRate / 12
        const termMonths = 12
        const firstPaymentFactor = 1 / termMonths + monthlyRate
        const maxPrincipal = Math.floor((profile.monthlyIncome * 0.35) / firstPaymentFactor)
        const maxUnits = Math.min(20, Math.floor(maxPrincipal / engine.financials.loanUnit))
        const loanProbability = Math.min(0.65, 0.08 + profile.creditNeed * 0.4)
        if (maxUnits >= 2 && chance(productRandom, loanProbability)) {
          const principal = integer(productRandom, 2, maxUnits) * engine.financials.loanUnit
          const basePrincipal = Math.floor(principal / termMonths)
          const estimatedThreePayments = Math.round(basePrincipal * 3 + principal * monthlyRate * 3)
          if (estimatedThreePayments <= hdbankAvailable) {
            const loanStart = addMonths(generatedAtDate, -4)
            let repaidPrincipal = 0
            let remainingPrincipal = principal
            const repaymentSchedule = Array.from({ length: termMonths }, (_, scheduleIndex) => {
              const installment = scheduleIndex + 1
              const principalPart = installment === termMonths ? principal - basePrincipal * 11 : basePrincipal
              const interest = Math.round(remainingPrincipal * monthlyRate)
              const dueDate = addMonths(loanStart, installment)
              const paid = installment <= 3
              const transferId = `HDB-FT-${suffix}-${installment}`
              if (paid) {
                const payment = principalPart + interest
                hdbankAvailable -= payment
                repaidPrincipal += principalPart
                hdbankFundsTransfers.push(
                  createTransfer({
                    id: transferId,
                    reference: `HDB-REPAY-${suffix}-${installment}`,
                    debtorAccount: hdbankAccountNumber,
                    creditorAccount: `HDB-LOAN-${suffix}`,
                    amount: payment,
                    type: "LOAN_REPAYMENT",
                    description: engine.repaymentDescription(installment),
                    bookedAt: dueDate,
                    recordedAt: generatedAt,
                    engine,
                  }),
                )
              }
              remainingPrincipal -= principalPart
              return {
                sequence: installment,
                dueDate: isoDate(dueDate),
                principal: money(principalPart, engine),
                interest: money(interest, engine),
                status: paid ? ("PAID" as const) : ("DUE" as const),
                paidAt: paid ? dueDate.toISOString() : undefined,
                fundsTransferId: paid ? transferId : undefined,
              }
            })
            loan = {
              application: "AA.ARRANGEMENT",
              header: recordHeader(`HDB-LOAN-${suffix}`, generatedAt),
              body: {
                customerId: hdbankCustomerId,
                arrangementId: `HDB-LOAN-${suffix}`,
                linkedSettlementAccount: hdbankAccountNumber,
                productLine: "LENDING",
                productGroup: "CONSUMER.LOANS",
                productId: "HDB.PERSONAL.LOAN",
                currency: engine.currency,
                originalPrincipal: money(principal, engine),
                outstandingPrincipal: money(principal - repaidPrincipal, engine),
                annualInterestRate: annualRate,
                termMonths,
                repaymentFrequency: "MONTHLY",
                startDate: isoDate(loanStart),
                maturityDate: isoDate(addMonths(loanStart, termMonths)),
                status: "CURRENT",
                repaymentSchedule,
              },
            }
          }
        }
      }
    }

    let vietjetCustomer: VietjetCustomer | undefined
    const bookings: VietjetBooking[] = []
    if (membership.has("vietjetAir")) {
      const person = observed.vietjetAir
      const customerId = sourceRecordIds.vietjetAir
      vietjetCustomer = {
        customerId,
        personId: person.personId,
        fullName: person.fullName,
        dateOfBirth: person.dateOfBirth,
        gender: person.gender,
        nationalId: person.nationalId,
        nationalityCode: engine.countryCode,
        phoneNumber: person.phoneNumber,
        emailAddress: person.emailAddress,
        preferredLanguage: engine.locale,
        skyJoyMemberNumber: `SJ${String(options.seed).padStart(8, "0").slice(-8)}${String(sequence).padStart(4, "0")}`,
      }
      const bookingRandom = randomFor(options.seed, sequence, "vietjet-bookings")
      const zeroProbability = Math.max(0.08, 0.55 - profile.travelPropensity * 0.45)
      const requestedBookings = chance(bookingRandom, zeroProbability)
        ? 0
        : Math.min(5, 1 + poisson(bookingRandom, 1.1 + profile.travelPropensity))
      const origin = engine.airportForProvince(canonical.address.province)
      const destinations = engine.destinations.filter((airport) => airport !== origin)
      const disposableIncome = Math.max(0, profile.monthlyIncome - profile.monthlyExpenses)

      for (let bookingIndex = 0; bookingIndex < requestedBookings; bookingIndex += 1) {
        const number = bookingIndex + 1
        const vikkiCap = wallet ? Math.min(vikkiAvailable * 0.25, disposableIncome * 0.35) : 0
        const hdbankCap = hdbankAccount ? Math.min(hdbankAvailable * 0.25, disposableIncome * 0.35) : 0
        const cashCap = Math.min(profile.liquidAssets * 0.12, disposableIncome * 0.35)
        const methods = [
          { value: "VIKKI_WALLET" as const, weight: vikkiCap >= engine.financials.booking[0] ? 4 : 0 },
          { value: "BANK_CARD" as const, weight: hdbankCap >= engine.financials.booking[0] ? 3 : 0 },
          { value: "CASH" as const, weight: cashCap >= engine.financials.booking[0] ? 1 : 0 },
        ]
        if (methods.every((method) => method.weight === 0)) break
        const method = weightedPick(bookingRandom, methods)
        const methodCap = method === "VIKKI_WALLET" ? vikkiCap : method === "BANK_CARD" ? hdbankCap : cashCap
        const maximumFare = Math.floor(Math.min(engine.financials.booking[1], methodCap))
        if (maximumFare < engine.financials.booking[0]) break
        const amount = integer(bookingRandom, engine.financials.booking[0], maximumFare)
        const reference = `${method === "VIKKI_WALLET" ? "VIK" : "HDB"}-VJ-${suffix}-${number}`
        const departure = addDays(generatedAtDate, -(330 - bookingIndex * 65))
        departure.setUTCHours(2 + bookingIndex * 3, 30, 0, 0)
        const arrival = new Date(departure.getTime() + 90 * 60 * 1_000)
        const paidAt = addDays(departure, -21)
        const referenceCode = bookingReference(options.seed, sequence, number)
        if (method === "VIKKI_WALLET") {
          vikkiAvailable -= amount
          vikkiFundsTransfers.push(
            createTransfer({
              id: `VIK-FT-${suffix}-VJ-${number}`,
              reference,
              debtorAccount: vikkiAccountNumber,
              creditorAccount: "VIETJET-AIR",
              amount,
              type: "WALLET_PAYMENT",
              description: engine.ticketDescription(referenceCode),
              bookedAt: paidAt,
              recordedAt: generatedAt,
              engine,
            }),
          )
        } else if (method === "BANK_CARD") {
          hdbankAvailable -= amount
          hdbankFundsTransfers.push(
            createTransfer({
              id: `HDB-FT-${suffix}-VJ-${number}`,
              reference,
              debtorAccount: hdbankAccountNumber,
              creditorAccount: "VIETJET-AIR",
              amount,
              type: "CARD_PAYMENT",
              description: engine.ticketDescription(referenceCode),
              bookedAt: paidAt,
              recordedAt: generatedAt,
              engine,
            }),
          )
        }
        bookings.push({
          bookingId: `VJ-B-${suffix}-${number}`,
          bookingReference: referenceCode,
          customerId,
          bookedAt: paidAt.toISOString(),
          status: "FLOWN",
          passengers: [{
            passengerId: `VJ-PAX-${suffix}-${number}`,
            fullName: person.fullName,
            passengerType: "ADULT",
          }],
          segments: [{
            flightNumber: `VJ${100 + sequence * 3 + bookingIndex}`,
            departureAirport: origin,
            arrivalAirport: destinations[bookingIndex % destinations.length] ?? "DAD",
            scheduledDepartureAt: departure.toISOString(),
            scheduledArrivalAt: arrival.toISOString(),
            fareClass: "ECO",
            seatNumber: `${integer(bookingRandom, 1, 30)}${pick(bookingRandom, ["A", "B", "C", "D", "E", "F"] as const)}`,
          }],
          totalAmount: money(amount, engine),
          payment: {
            paymentId: `VJ-PAY-${suffix}-${number}`,
            method,
            status: "PAID",
            paidAt: paidAt.toISOString(),
            externalTransactionReference: method === "CASH" ? undefined : reference,
          },
        })
      }
    }

    if (vikkiAccount) setAccountBalance(vikkiAccount, vikkiAvailable, engine)
    if (hdbankAccount) setAccountBalance(hdbankAccount, hdbankAvailable, engine)

    let crmContact: CrmContact | undefined
    let crmPollBatch: CrmPollBatch | undefined
    let interactions: CrmCampaignInteraction[] = []
    if (membership.has("crm")) {
      const person = observed.crm
      const crmRandom = randomFor(options.seed, sequence, "crm-activity")
      crmContact = {
        contactId: sourceRecordIds.crm,
        personId: person.personId,
        sourceRecordId: `CRM-SOURCE-${suffix}`,
        fullName: person.fullName,
        fullNameAscii: ascii(person.fullName),
        dateOfBirth: person.dateOfBirth,
        nationalId: person.nationalId,
        phoneNumber: person.phoneNumber,
        emailAddress: person.emailAddress,
        preferredLanguage: engine.locale,
        countryCode: engine.countryCode,
        address: person.address,
        lifecycleStage: membership.size > 1 ? "CUSTOMER" : "PROSPECT",
        status: "ACTIVE",
        createdAt: addDays(generatedAtDate, -integer(crmRandom, 90, 900)).toISOString(),
        updatedAt: generatedAt,
      }
      interactions = Array.from({ length: Math.min(4, poisson(crmRandom, 1.3)) }, (_, interactionIndex) => ({
        interactionId: `CRM-I-${suffix}-${interactionIndex + 1}`,
        contactId: crmContact!.contactId,
        campaignId: `CAMPAIGN-${region.toUpperCase()}-${interactionIndex + 1}`,
        campaignName: region === "vn" ? `Chiến dịch khách hàng ${interactionIndex + 1}` : `แคมเปญลูกค้า ${interactionIndex + 1}`,
        channel: pick(crmRandom, ["EMAIL", "SMS", "PUSH_NOTIFICATION", "PHONE_CALL"] as const),
        direction: "OUTBOUND" as const,
        outcome: pick(crmRandom, ["DELIVERED", "OPENED", "CLICKED", "RESPONDED", "NO_ANSWER"] as const),
        occurredAt: addDays(generatedAtDate, -integer(crmRandom, 1, 60)).toISOString(),
      }))
      crmPollBatch = {
        pollId: `CRM-POLL-${suffix}`,
        startedAt: generatedAt,
        completedAt: new Date(generatedAtDate.getTime() + 2_000).toISOString(),
        cursor: crmContact.sourceRecordId,
        status: "COMPLETED",
        recordsRead: 1,
      }
    }

    if (emitted.has("vikkiBank") && vikkiCustomer) {
      record.vikkiBank = { customer: vikkiCustomer, account: vikkiAccount, wallet, fundsTransfers: vikkiFundsTransfers }
      coverage.vikkiBank += 1
      vikkiCustomers.push(vikkiCustomer)
      if (vikkiAccount) vikkiAccounts.push(vikkiAccount)
      if (wallet) vikkiWallets.push(wallet)
      vikkiTransfers.push(...vikkiFundsTransfers)
    }
    if (emitted.has("hdbank") && hdbankCustomer) {
      record.hdbank = { customer: hdbankCustomer, account: hdbankAccount, loan, fundsTransfers: hdbankFundsTransfers }
      coverage.hdbank += 1
      hdbankCustomers.push(hdbankCustomer)
      if (hdbankAccount) hdbankAccounts.push(hdbankAccount)
      if (loan) hdbankLoans.push(loan)
      hdbankTransfers.push(...hdbankFundsTransfers)
    }
    if (emitted.has("vietjetAir") && vietjetCustomer) {
      record.vietjetAir = { customer: vietjetCustomer, bookings }
      coverage.vietjetAir += 1
      vietjetCustomers.push(vietjetCustomer)
      vietjetBookings.push(...bookings)
    }
    if (emitted.has("crm") && crmContact && crmPollBatch) {
      record.crm = { contact: crmContact, campaignInteractions: interactions, pollBatch: crmPollBatch }
      coverage.crm += 1
      crmContacts.push(crmContact)
      crmInteractions.push(...interactions)
      crmPollBatches.push(crmPollBatch)
    }
    records.push(record)
  }

  const dataset = SyntheticDatasetSchema.parse({
    schemaVersion: "2.0.0",
    region,
    generatedAt,
    seed: options.seed,
    people,
    organizations: {
      vikkiBank: {
        organization: {
          organizationId: "vikki-bank",
          legalName: "Ngân hàng TNHH MTV Số Vikki",
          countryCode: engine.countryCode,
          timezone: engine.timezone,
        },
        sourceSystem: "TEMENOS_T24",
        customers: vikkiCustomers,
        accounts: vikkiAccounts,
        wallets: vikkiWallets,
        fundsTransfers: vikkiTransfers,
      },
      hdbank: {
        organization: {
          organizationId: "hdbank",
          legalName: "Ngân hàng TMCP Phát triển Thành phố Hồ Chí Minh",
          countryCode: engine.countryCode,
          timezone: engine.timezone,
        },
        sourceSystem: "TEMENOS_T24",
        customers: hdbankCustomers,
        accounts: hdbankAccounts,
        loans: hdbankLoans,
        fundsTransfers: hdbankTransfers,
      },
      crm: {
        organization: {
          organizationId: "group-crm",
          legalName: "Sovico Group CRM",
          countryCode: engine.countryCode,
          timezone: engine.timezone,
        },
        sourceSystem: "GROUP_CRM",
        pollBatches: crmPollBatches,
        contacts: crmContacts,
        campaignInteractions: crmInteractions,
      },
      vietjetAir: {
        organization: {
          organizationId: "vietjet-air",
          legalName: "Công ty Cổ phần Hàng không VietJet",
          countryCode: engine.countryCode,
          timezone: engine.timezone,
        },
        sourceSystem: "VIETJET_RESERVATION",
        customers: vietjetCustomers,
        bookings: vietjetBookings,
      },
    },
  })

  return { dataset, records, manifest, coverage }
}
