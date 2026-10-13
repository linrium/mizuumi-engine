import { createHash } from "node:crypto"
import type { IncrementalEvent } from "./events.js"
import {
  generateSyntheticData,
  sourceNames,
  type AnomalyManifestEntry,
  type GeneratedPersonRecords,
  type ScenarioName,
  type SourceMode,
  type SourceName,
} from "./generator.js"
import type { RegionCode } from "./generators/index.js"
import type { Random } from "./generators/types.js"
import { chance, integer, pick, poisson } from "./generators/utils.js"
import type { T24FundsTransfer } from "./schemas/temenos-t24.js"
import type { VietjetBooking } from "./schemas/organizations/vietjet-air.js"
import type { IncrementalStore } from "./incremental-store.js"

export type TemporalAnomaly = AnomalyManifestEntry & {
  introducedAt: string
  correctedAt?: string
  status: "ACTIVE" | "CORRECTED"
}

type PendingIdentityUpdate = {
  truthId: string
  source: SourceName
  field: "phoneNumber" | "emailAddress"
  value: string
  dueAt: string
}

type StatefulPerson = {
  sequence: number
  record: GeneratedPersonRecords
  versions: Partial<Record<SourceName, number>>
  publishedSources: SourceName[]
  publishedVersions: Partial<Record<SourceName, number>>
}

export type IncrementalStreamState = {
  streamId: string
  region: RegionCode
  seed: number
  scenario: ScenarioName
  sourceMode: SourceMode
  anomalyRate?: number
  maxAnomaliesPerPerson: number
  newPeoplePerDay: number
  createdAt: string
  cursor: string
  tick: number
  nextSequence: number
  people: StatefulPerson[]
  anomalies: TemporalAnomaly[]
  pendingIdentityUpdates: PendingIdentityUpdate[]
}

export type InitializeIncrementalOptions = {
  streamId: string
  region: RegionCode
  asOf: Date
  initialPopulation: number
  seed: number
  scenario: ScenarioName
  sourceMode: SourceMode
  sources?: SourceName[]
  anomalyRate?: number
  maxAnomaliesPerPerson: number
  newPeoplePerDay: number
  maxEvents: number
}

export type AdvanceIncrementalOptions = {
  streamId: string
  asOf: Date
  sources?: SourceName[]
  newPeople?: number
  emitCurrentRecord?: boolean
  maxEvents: number
}

export type IncrementalGenerationResult = {
  state: IncrementalStreamState
  previousCursor?: string
  events: IncrementalEvent[]
  newPeople: number
  idempotent: boolean
}

export class IncrementalRequestError extends Error {}
export class IncrementalEventLimitError extends Error {}

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

function randomFor(seed: number, ...parts: (string | number)[]) {
  const digest = createHash("sha256").update([seed, ...parts].join("|")).digest()
  return mulberry32(digest.readUInt32LE(0))
}

function stableEventId(...parts: (string | number)[]) {
  const hex = createHash("sha256").update(parts.join("|")).digest("hex").slice(0, 32)
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-5${hex.slice(13, 16)}-a${hex.slice(17, 20)}-${hex.slice(20)}`
}

function sourceSlug(source: SourceName) {
  return source === "vikkiBank" ? "vikki" : source === "vietjetAir" ? "vietjetair" : source
}

function sourceRecord(record: GeneratedPersonRecords, source: SourceName) {
  return record[source]
}

function sourceEntityId(record: GeneratedPersonRecords, source: SourceName) {
  if (source === "hdbank") return record.hdbank?.customer.header.id
  if (source === "vikkiBank") return record.vikkiBank?.customer.header.id
  if (source === "vietjetAir") return record.vietjetAir?.customer.customerId
  return record.crm?.contact.contactId
}

function makeEvent(input: {
  state: IncrementalStreamState
  person: StatefulPerson
  source: SourceName
  operation: "CREATE" | "UPDATE" | "DELETE"
  changedFields: string[]
  at: string
  ordinal: number
}): IncrementalEvent {
  const record = sourceRecord(input.person.record, input.source)
  const entityId = sourceEntityId(input.person.record, input.source)
  if (!record || !entityId) throw new Error(`Missing ${input.source} source record`)
  const version = input.person.versions[input.source] ?? 1
  const slug = sourceSlug(input.source)
  return {
    id: stableEventId(
      input.state.streamId,
      input.state.tick,
      input.person.sequence,
      input.source,
      version,
      input.operation,
      input.ordinal,
    ),
    streamId: input.state.streamId,
    region: input.state.region,
    source: input.source,
    subject: entityId,
    type: `${input.state.region}.mizuumi.synthetic.${slug}.customer.${input.operation.toLowerCase()}`,
    time: input.at,
    data: {
      operation: input.operation,
      entityType: "CUSTOMER",
      entityId,
      entityVersion: version,
      effectiveAt: input.at,
      observedAt: input.at,
      changedFields: input.changedFields,
      record: structuredClone(record),
    },
  }
}

function presentSources(record: GeneratedPersonRecords) {
  return sourceNames.filter((source) => sourceRecord(record, source) !== undefined)
}

function initialStatefulPerson(record: GeneratedPersonRecords, sequence: number): StatefulPerson {
  return {
    sequence,
    record,
    versions: Object.fromEntries(presentSources(record).map((source) => [source, 1])),
    publishedSources: [],
    publishedVersions: {},
  }
}

function enforceEventLimit(events: IncrementalEvent[], maximum: number) {
  if (events.length > maximum) {
    throw new IncrementalEventLimitError(
      `Generated ${events.length} events, exceeding maxEvents=${maximum}`,
    )
  }
}

function scenarioRate(scenario: ScenarioName) {
  return { clean: 0, realistic: 0.12, "matcher-demo": 0.35, stress: 0.6 }[scenario]
}

function hazardProbability(ratePerDay: number, elapsedDays: number) {
  return 1 - Math.exp(-Math.max(0, ratePerDay) * Math.max(0, elapsedDays))
}

function identityValue(
  person: StatefulPerson,
  source: SourceName,
  field: "fullName" | "phoneNumber" | "emailAddress",
) {
  if (source === "hdbank" || source === "vikkiBank") {
    const customer = person.record[source]!.customer
    if (field === "fullName") return customer.body.displayNames[0]!.displayName
    if (field === "phoneNumber") return customer.body.communicationDevices[0]!.phoneNumber
    return customer.body.communicationDevices[0]!.email
  }
  if (source === "vietjetAir") return person.record.vietjetAir!.customer[field]
  return person.record.crm!.contact[field]
}

function setIdentityValue(
  person: StatefulPerson,
  source: SourceName,
  field: "fullName" | "phoneNumber" | "emailAddress",
  value: string,
) {
  if (source === "hdbank" || source === "vikkiBank") {
    const sourceRecord = person.record[source]!
    if (field === "fullName") {
      sourceRecord.customer.body.displayNames[0]!.displayName = value
      sourceRecord.customer.body.customerNames[0]!.customerName = value
      if (sourceRecord.account) {
        sourceRecord.account.body.accountTitle = value
        sourceRecord.account.body.shortTitle = value
      }
    } else if (field === "phoneNumber") {
      sourceRecord.customer.body.communicationDevices[0]!.phoneNumber = value
    } else {
      sourceRecord.customer.body.communicationDevices[0]!.email = value
    }
    return
  }
  if (source === "vietjetAir") {
    person.record.vietjetAir!.customer[field] = value
    if (field === "fullName") {
      for (const booking of person.record.vietjetAir!.bookings) {
        for (const passenger of booking.passengers) passenger.fullName = value
      }
    }
    return
  }
  person.record.crm!.contact[field] = value
  if (field === "fullName") person.record.crm!.contact.fullNameAscii = value
}

function mutatePhone(phone: string, random: Random) {
  const index = integer(random, Math.max(3, phone.length - 5), phone.length - 1)
  const next = (Number(phone[index]) + integer(random, 1, 9)) % 10
  return `${phone.slice(0, index)}${next}${phone.slice(index + 1)}`
}

function mutateEmail(email: string, suffix: string) {
  const [local, domain] = email.split("@") as [string, string]
  return `${local}.${suffix}@${domain}`
}

function incrementVersion(person: StatefulPerson, source: SourceName) {
  person.versions[source] = (person.versions[source] ?? 1) + 1
}

function addUpdateEvent(
  events: IncrementalEvent[],
  state: IncrementalStreamState,
  person: StatefulPerson,
  source: SourceName,
  selected: Set<SourceName>,
  changedFields: string[],
  at: string,
) {
  if (!selected.has(source) || !person.publishedSources.includes(source)) return
  events.push(
    makeEvent({
      state,
      person,
      source,
      operation: "UPDATE",
      changedFields,
      at,
      ordinal: events.length,
    }),
  )
  person.publishedVersions[source] = person.versions[source]
}

function initializeEvents(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  at: string,
) {
  const events: IncrementalEvent[] = []
  for (const person of state.people) {
    for (const source of presentSources(person.record)) {
      if (!selected.has(source)) continue
      events.push(
        makeEvent({
          state,
          person,
          source,
          operation: "CREATE",
          changedFields: ["*"],
          at,
          ordinal: events.length,
        }),
      )
      person.publishedSources.push(source)
      person.publishedVersions[source] = person.versions[source]
    }
  }
  return events
}

export function initializeIncrementalStream(
  store: IncrementalStore,
  options: InitializeIncrementalOptions,
): IncrementalGenerationResult {
  const at = options.asOf.toISOString()
  const generated = generateSyntheticData({
    region: options.region,
    count: options.initialPopulation,
    seed: options.seed,
    generatedAt: options.asOf,
    scenario: options.scenario,
    sourceMode: options.sourceMode,
    anomalyRate: options.anomalyRate,
    maxAnomaliesPerPerson: options.maxAnomaliesPerPerson,
  })
  const state: IncrementalStreamState = {
    streamId: options.streamId,
    region: options.region,
    seed: options.seed,
    scenario: options.scenario,
    sourceMode: options.sourceMode,
    anomalyRate: options.anomalyRate,
    maxAnomaliesPerPerson: options.maxAnomaliesPerPerson,
    newPeoplePerDay: options.newPeoplePerDay,
    createdAt: at,
    cursor: at,
    tick: 0,
    nextSequence: options.initialPopulation + 1,
    people: generated.records.map((record, index) => initialStatefulPerson(record, index + 1)),
    anomalies: generated.manifest.map((anomaly) => ({
      ...anomaly,
      introducedAt: at,
      status: "ACTIVE",
    })),
    pendingIdentityUpdates: [],
  }
  const selected = new Set(options.sources ?? sourceNames)
  const events = initializeEvents(state, selected, at)
  enforceEventLimit(events, options.maxEvents)
  store.createStream(state, events)
  return { state, events, newPeople: options.initialPopulation, idempotent: false }
}

function createNewPeople(
  state: IncrementalStreamState,
  count: number,
  at: Date,
) {
  if (count === 0) return []
  const generated = generateSyntheticData({
    region: state.region,
    count,
    seed: state.seed,
    sequenceStart: state.nextSequence - 1,
    generatedAt: at,
    scenario: state.scenario,
    sourceMode: state.sourceMode,
    anomalyRate: state.anomalyRate,
    maxAnomaliesPerPerson: state.maxAnomaliesPerPerson,
  })
  const records = generated.records
  const truthIds = new Set(records.map((record) => record.person.personId))
  state.anomalies.push(
    ...generated.manifest
      .filter((anomaly) => truthIds.has(anomaly.truthId))
      .map((anomaly) => ({
        ...anomaly,
        introducedAt: at.toISOString(),
        status: "ACTIVE" as const,
      })),
  )
  const people = records.map((record, index) =>
    initialStatefulPerson(record, state.nextSequence + index),
  )
  state.people.push(...people)
  state.nextSequence += people.length
  return people
}

function addDays(date: Date, days: number) {
  return new Date(date.getTime() + days * 86_400_000)
}

function bookingReference(state: IncrementalStreamState, person: StatefulPerson, number: number) {
  return createHash("sha256")
    .update(`${state.streamId}|${state.tick}|${person.sequence}|${number}`)
    .digest("hex")
    .slice(0, 6)
    .toUpperCase()
}

function generateProductAdoption(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const person of state.people) {
    const bank = person.record.vikkiBank
    if (!bank?.account || bank.wallet) continue
    const random = randomFor(state.seed, state.tick, person.sequence, "wallet-adoption")
    const dailyRate = 0.004 * person.record.profile.digitalAffinity
    if (!chance(random, hazardProbability(dailyRate, elapsedDays))) continue
    bank.wallet = {
      walletId: `VIK-W-${state.region}-${state.seed}-${String(person.sequence).padStart(4, "0")}`,
      customerId: bank.customer.header.id,
      settlementAccountNumber: bank.account.body.accountNumber,
      walletType: "VIKKI_DIGITAL_WALLET",
      activatedAt: at.slice(0, 10),
      status: "ACTIVE",
    }
    incrementVersion(person, "vikkiBank")
    addUpdateEvent(events, state, person, "vikkiBank", selected, ["wallet"], at)
  }
}

function generateTravelActivity(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const person of state.people) {
    const vietjet = person.record.vietjetAir
    if (!vietjet) continue
    const random = randomFor(state.seed, state.tick, person.sequence, "travel")
    const dailyRate = (0.25 + person.record.profile.travelPropensity) / 120
    if (!chance(random, hazardProbability(dailyRate, elapsedDays))) continue

    const disposable = Math.max(0, person.record.profile.monthlyIncome - person.record.profile.monthlyExpenses)
    const minimum = state.region === "vn" ? 900_000 : 1_200
    const marketMaximum = state.region === "vn" ? 2_500_000 : 4_500
    const vikki = person.record.vikkiBank
    const hdbank = person.record.hdbank
    const methods = [
      { source: "vikkiBank" as const, method: "VIKKI_WALLET" as const, bank: vikki, eligible: Boolean(vikki?.wallet) },
      { source: "hdbank" as const, method: "BANK_CARD" as const, bank: hdbank, eligible: true },
    ].filter((candidate) => candidate.eligible && candidate.bank?.account)
    const payable = methods.filter((candidate) => {
      const balance = accountBalance(candidate.bank!.account)
      return Math.min(balance * 0.25, disposable * 0.35, marketMaximum) >= minimum
    })
    const cashCap = Math.min(person.record.profile.liquidAssets * 0.12, disposable * 0.35, marketMaximum)
    if (payable.length === 0 && cashCap < minimum) continue

    const chosen = payable.length > 0 && chance(random, 0.85) ? pick(random, payable) : undefined
    const maximum = Math.floor(chosen
      ? Math.min(accountBalance(chosen.bank!.account) * 0.25, disposable * 0.35, marketMaximum)
      : cashCap)
    if (maximum < minimum) continue
    const amount = integer(random, minimum, maximum)
    const number = vietjet.bookings.length + 1
    const referenceCode = bookingReference(state, person, number)
    const departure = addDays(new Date(at), integer(random, 14, 90))
    departure.setUTCHours(integer(random, 1, 20), 30, 0, 0)
    const arrival = new Date(departure.getTime() + integer(random, 60, 150) * 60_000)
    const routes = state.region === "vn"
      ? [["SGN", "HAN"], ["HAN", "DAD"], ["SGN", "CXR"]]
      : [["BKK", "CNX"], ["BKK", "HKT"], ["BKK", "KBV"]]
    const route = pick(random, routes)
    const externalReference = chosen
      ? `${chosen.source === "vikkiBank" ? "VIK" : "HDB"}-VJ-${state.streamId}-${state.tick}-${person.sequence}-${number}`
      : undefined
    const booking: VietjetBooking = {
      bookingId: `VJ-B-${state.streamId}-${person.sequence}-${number}`,
      bookingReference: referenceCode,
      customerId: vietjet.customer.customerId,
      bookedAt: at,
      status: "CONFIRMED",
      passengers: [{
        passengerId: `VJ-PAX-${state.streamId}-${person.sequence}-${number}`,
        fullName: vietjet.customer.fullName,
        passengerType: "ADULT",
      }],
      segments: [{
        flightNumber: `VJ${100 + ((person.sequence + number + state.tick) % 900)}`,
        departureAirport: route[0]!,
        arrivalAirport: route[1]!,
        scheduledDepartureAt: departure.toISOString(),
        scheduledArrivalAt: arrival.toISOString(),
        fareClass: "ECO",
        seatNumber: `${integer(random, 1, 30)}${pick(random, ["A", "B", "C", "D", "E", "F"] as const)}`,
      }],
      totalAmount: { amount, currency: person.record.profile.currency },
      payment: {
        paymentId: `VJ-PAY-${state.streamId}-${person.sequence}-${number}`,
        method: chosen?.method ?? "CASH",
        status: "PAID",
        paidAt: at,
        externalTransactionReference: externalReference,
      },
    }
    vietjet.bookings.push(booking)
    incrementVersion(person, "vietjetAir")
    addUpdateEvent(events, state, person, "vietjetAir", selected, ["bookings"], at)

    if (chosen?.bank?.account && externalReference) {
      const balance = accountBalance(chosen.bank.account)
      const transfer = createIncrementalTransfer({
        id: `${chosen.source === "vikkiBank" ? "VIK" : "HDB"}-FT-${externalReference}`,
        reference: externalReference,
        debtorAccount: chosen.bank.account.body.accountNumber,
        amount,
        currency: person.record.profile.currency,
        type: chosen.source === "vikkiBank" ? "WALLET_PAYMENT" : "CARD_PAYMENT",
        at,
      })
      transfer.body.creditorAccount = "VIETJET-AIR"
      transfer.body.remittanceInformation = `VietJet ticket ${referenceCode}`
      chosen.bank.fundsTransfers.push(transfer)
      setBalance(chosen.bank.account, balance - amount)
      incrementVersion(person, chosen.source)
      addUpdateEvent(events, state, person, chosen.source, selected, ["fundsTransfers", "account.workingBalance"], at)
    }
  }
}

function applyPendingIdentityUpdates(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  at: string,
) {
  const remaining: PendingIdentityUpdate[] = []
  for (const pending of state.pendingIdentityUpdates) {
    if (pending.dueAt > at) {
      remaining.push(pending)
      continue
    }
    const person = state.people.find((candidate) => candidate.record.person.personId === pending.truthId)
    if (!person || !sourceRecord(person.record, pending.source)) continue
    setIdentityValue(person, pending.source, pending.field, pending.value)
    incrementVersion(person, pending.source)
    addUpdateEvent(events, state, person, pending.source, selected, [pending.field], at)
  }
  state.pendingIdentityUpdates = remaining
}

function correctAnomalies(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const anomaly of state.anomalies) {
    if (anomaly.status !== "ACTIVE") continue
    const random = randomFor(state.seed, state.tick, anomaly.truthId, anomaly.source, anomaly.field, "correct")
    if (!chance(random, hazardProbability(1 / 5, elapsedDays))) continue
    const person = state.people.find((candidate) => candidate.record.person.personId === anomaly.truthId)
    if (!person || !["fullName", "phoneNumber", "emailAddress"].includes(anomaly.field)) continue
    const field = anomaly.field as "fullName" | "phoneNumber" | "emailAddress"
    const canonical = String(person.record.person[field])
    setIdentityValue(person, anomaly.source, field, canonical)
    incrementVersion(person, anomaly.source)
    anomaly.status = "CORRECTED"
    anomaly.correctedAt = at
    addUpdateEvent(events, state, person, anomaly.source, selected, [field], at)
  }
}

function createCanonicalChanges(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const person of state.people) {
    const random = randomFor(state.seed, state.tick, person.sequence, "canonical-change")
    if (!chance(random, hazardProbability(0.12 / 365, elapsedDays))) continue
    const field = pick(random, ["phoneNumber", "emailAddress"] as const)
    const value =
      field === "phoneNumber"
        ? mutatePhone(person.record.person.phoneNumber, random)
        : mutateEmail(person.record.person.emailAddress, `v${state.tick}`)
    person.record.person[field] = value
    const sources = presentSources(person.record)
    if (sources.length === 0) continue
    const origin = sources.includes("crm") ? "crm" : pick(random, sources)
    setIdentityValue(person, origin, field, value)
    incrementVersion(person, origin)
    addUpdateEvent(events, state, person, origin, selected, [field], at)
    for (const source of sources) {
      if (source === origin) continue
      const delayDays = integer(random, 1, 14)
      state.pendingIdentityUpdates.push({
        truthId: person.record.person.personId,
        source,
        field,
        value,
        dueAt: new Date(new Date(at).getTime() + delayDays * 86_400_000).toISOString(),
      })
    }
  }
}

function introduceAnomalies(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  const targetMonthlyRate = state.anomalyRate ?? scenarioRate(state.scenario)
  if (targetMonthlyRate <= 0) return
  const dailyRate = -Math.log(1 - Math.min(0.99, targetMonthlyRate)) / 30
  for (const person of state.people) {
    const random = randomFor(state.seed, state.tick, person.sequence, "incremental-anomaly")
    if (!chance(random, hazardProbability(dailyRate, elapsedDays))) continue
    const activeCount = state.anomalies.filter(
      (anomaly) => anomaly.status === "ACTIVE" && anomaly.truthId === person.record.person.personId,
    ).length
    if (activeCount >= state.maxAnomaliesPerPerson) continue
    const sources = presentSources(person.record)
    if (sources.length === 0) continue
    const source = pick(random, sources)
    const field = pick(random, ["phoneNumber", "emailAddress", "fullName"] as const)
    if (
      state.anomalies.some(
        (anomaly) =>
          anomaly.status === "ACTIVE" &&
          anomaly.truthId === person.record.person.personId &&
          anomaly.source === source &&
          anomaly.field === field,
      )
    ) {
      continue
    }
    const canonicalValue = String(person.record.person[field])
    const observedValue =
      field === "phoneNumber"
        ? mutatePhone(canonicalValue, random)
        : field === "emailAddress"
          ? mutateEmail(canonicalValue, `err${state.tick}`)
          : `${canonicalValue.slice(0, -1)}${canonicalValue.at(-1) === "a" ? "e" : "a"}`
    setIdentityValue(person, source, field, observedValue)
    incrementVersion(person, source)
    state.anomalies.push({
      truthId: person.record.person.personId,
      source,
      sourceRecordId: sourceEntityId(person.record, source)!,
      field,
      kind: "typo",
      canonicalValue,
      observedValue,
      severity: "low",
      introducedAt: at,
      status: "ACTIVE",
    })
    addUpdateEvent(events, state, person, source, selected, [field], at)
  }
}

function accountBalance(record: NonNullable<GeneratedPersonRecords["hdbank"]>["account"]) {
  return record?.body.workingBalance.amount ?? 0
}

function setBalance(
  account: NonNullable<NonNullable<GeneratedPersonRecords["hdbank"]>["account"]>,
  amount: number,
) {
  account.body.onlineActualBalance.amount = amount
  account.body.onlineClearedBalance.amount = amount
  account.body.workingBalance.amount = amount
}

function createIncrementalTransfer(input: {
  id: string
  reference: string
  debtorAccount: string
  amount: number
  currency: "VND" | "THB"
  type: T24FundsTransfer["body"]["transactionType"]
  at: string
}): T24FundsTransfer {
  const date = input.at.slice(0, 10)
  return {
    application: "FUNDS.TRANSFER",
    header: {
      id: input.id,
      status: "success",
      transactionStatus: "Live",
      versionNumber: "1",
      recordedAt: input.at,
    },
    body: {
      transactionReference: input.reference,
      debtorAccount: input.debtorAccount,
      creditorAccount: "MERCHANT-INCREMENTAL",
      transactionAmount: { amount: input.amount, currency: input.currency },
      transactionType: input.type,
      remittanceInformation: "Incremental synthetic payment",
      bookingDate: date,
      valueDate: date,
      paymentStatus: "COMPLETED",
    },
  }
}

function generateBankActivity(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const person of state.people) {
    for (const source of ["hdbank", "vikkiBank"] as const) {
      const bank = person.record[source]
      if (!bank?.account) continue
      const random = randomFor(state.seed, state.tick, person.sequence, source, "payments")
      const lambda = (person.record.profile.segment === "low-activity" ? 0.05 : 0.18) * elapsedDays
      const count = Math.min(3, poisson(random, lambda))
      for (let index = 0; index < count; index += 1) {
        const balance = accountBalance(bank.account)
        const disposable = Math.max(
          0,
          person.record.profile.monthlyIncome - person.record.profile.monthlyExpenses,
        )
        const minimum = state.region === "vn" ? 50_000 : 50
        const maximum = Math.floor(Math.min(balance * 0.1, disposable * 0.2))
        if (maximum < minimum) break
        const amount = integer(random, minimum, maximum)
        const reference = `INC-${state.streamId}-${state.tick}-${person.sequence}-${source}-${index + 1}`
        bank.fundsTransfers.push(
          createIncrementalTransfer({
            id: `${source === "hdbank" ? "HDB" : "VIK"}-FT-${reference}`,
            reference,
            debtorAccount: bank.account.body.accountNumber,
            amount,
            currency: person.record.profile.currency,
            type: source === "vikkiBank" && person.record.vikkiBank?.wallet
              ? "WALLET_PAYMENT"
              : "ACCOUNT_TRANSFER",
            at,
          }),
        )
        setBalance(bank.account, balance - amount)
        incrementVersion(person, source)
        addUpdateEvent(
          events,
          state,
          person,
          source,
          selected,
          ["fundsTransfers", "account.workingBalance"],
          at,
        )
      }
    }
  }
}

function processLoanRepayments(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  at: string,
) {
  for (const person of state.people) {
    const bank = person.record.hdbank
    if (!bank?.account || !bank.loan) continue
    let changed = false
    for (const installment of bank.loan.body.repaymentSchedule) {
      if (installment.status !== "DUE" || installment.dueDate > at.slice(0, 10)) continue
      const payment = installment.principal.amount + installment.interest.amount
      const balance = accountBalance(bank.account)
      if (payment > balance) {
        installment.status = "OVERDUE"
        changed = true
        continue
      }
      const reference = `HDB-REPAY-${state.streamId}-${state.tick}-${person.sequence}-${installment.sequence}`
      const transfer = createIncrementalTransfer({
        id: `HDB-FT-${reference}`,
        reference,
        debtorAccount: bank.account.body.accountNumber,
        amount: payment,
        currency: person.record.profile.currency,
        type: "LOAN_REPAYMENT",
        at,
      })
      transfer.body.creditorAccount = bank.loan.body.arrangementId
      bank.fundsTransfers.push(transfer)
      setBalance(bank.account, balance - payment)
      installment.status = "PAID"
      installment.paidAt = at
      installment.fundsTransferId = transfer.header.id
      bank.loan.body.outstandingPrincipal.amount = Math.max(
        0,
        bank.loan.body.outstandingPrincipal.amount - installment.principal.amount,
      )
      changed = true
    }
    if (changed) {
      incrementVersion(person, "hdbank")
      addUpdateEvent(
        events,
        state,
        person,
        "hdbank",
        selected,
        ["loan.repaymentSchedule", "account.workingBalance"],
        at,
      )
    }
  }
}

function generateCrmActivity(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  elapsedDays: number,
  at: string,
) {
  for (const person of state.people) {
    const crm = person.record.crm
    if (!crm) continue
    const random = randomFor(state.seed, state.tick, person.sequence, "crm-interaction")
    if (!chance(random, hazardProbability(0.04, elapsedDays))) continue
    const sequence = crm.campaignInteractions.length + 1
    crm.campaignInteractions.push({
      interactionId: `CRM-I-${state.streamId}-${person.sequence}-${state.tick}-${sequence}`,
      contactId: crm.contact.contactId,
      campaignId: `CAMPAIGN-${state.region.toUpperCase()}-${state.tick}`,
      campaignName: state.region === "vn" ? "Chăm sóc khách hàng" : "ดูแลลูกค้า",
      channel: pick(random, ["EMAIL", "SMS", "PUSH_NOTIFICATION", "PHONE_CALL"] as const),
      direction: "OUTBOUND",
      outcome: pick(random, ["DELIVERED", "OPENED", "CLICKED", "RESPONDED", "NO_ANSWER"] as const),
      occurredAt: at,
    })
    crm.contact.updatedAt = at
    incrementVersion(person, "crm")
    addUpdateEvent(events, state, person, "crm", selected, ["campaignInteractions"], at)
  }
}

function processFlightStatus(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  at: string,
) {
  for (const person of state.people) {
    const vietjet = person.record.vietjetAir
    if (!vietjet) continue
    let changed = false
    for (const booking of vietjet.bookings) {
      if (
        booking.status === "CONFIRMED" &&
        booking.segments.every((segment) => segment.scheduledArrivalAt <= at)
      ) {
        booking.status = "FLOWN"
        changed = true
      }
    }
    if (changed) {
      incrementVersion(person, "vietjetAir")
      addUpdateEvent(events, state, person, "vietjetAir", selected, ["bookings.status"], at)
    }
  }
}

function flushSelectedSources(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  at: string,
) {
  for (const person of state.people) {
    for (const source of presentSources(person.record)) {
      if (!selected.has(source) || person.publishedSources.includes(source)) continue
      events.push(
        makeEvent({
          state,
          person,
          source,
          operation: "CREATE",
          changedFields: ["*"],
          at,
          ordinal: events.length,
        }),
      )
      person.publishedSources.push(source)
      person.publishedVersions[source] = person.versions[source]
      continue
    }
    for (const source of presentSources(person.record)) {
      if (!selected.has(source) || !person.publishedSources.includes(source)) continue
      if ((person.publishedVersions[source] ?? 0) >= (person.versions[source] ?? 1)) continue
      events.push(
        makeEvent({
          state,
          person,
          source,
          operation: "UPDATE",
          changedFields: ["*"],
          at,
          ordinal: events.length,
        }),
      )
      person.publishedVersions[source] = person.versions[source]
    }
  }
}

function emitAndCoalesceCurrentRecords(
  state: IncrementalStreamState,
  selected: Set<SourceName>,
  events: IncrementalEvent[],
  at: string,
) {
  for (const person of state.people) {
    for (const source of presentSources(person.record)) {
      if (!selected.has(source) || !person.publishedSources.includes(source)) continue
      const subject = sourceEntityId(person.record, source)
      if (events.some((event) => event.source === source && event.subject === subject)) continue
      events.push(
        makeEvent({
          state,
          person,
          source,
          operation: "UPDATE",
          changedFields: [],
          at,
          ordinal: events.length,
        }),
      )
    }
  }

  const latestByRecord = new Map<string, IncrementalEvent>()
  for (const event of events) latestByRecord.set(`${event.source}|${event.subject}`, event)
  events.splice(0, events.length, ...latestByRecord.values())
}

export function advanceIncrementalStream(
  store: IncrementalStore,
  options: AdvanceIncrementalOptions,
): IncrementalGenerationResult {
  const stored = store.getStream(options.streamId)
  if (!stored) throw new IncrementalRequestError(`Unknown stream: ${options.streamId}`)
  const target = options.asOf.toISOString()
  if (target < stored.cursor) {
    throw new IncrementalRequestError(
      `asOf ${target} is earlier than stream cursor ${stored.cursor}`,
    )
  }
  if (target === stored.cursor) {
    return { state: stored, events: [], newPeople: 0, idempotent: true }
  }

  const state = structuredClone(stored)
  const previousCursor = state.cursor
  const elapsedDays =
    (options.asOf.getTime() - new Date(previousCursor).getTime()) / 86_400_000
  state.cursor = target
  state.tick += 1
  const selected = new Set(options.sources ?? sourceNames)
  const events: IncrementalEvent[] = []

  applyPendingIdentityUpdates(state, selected, events, target)
  correctAnomalies(state, selected, events, elapsedDays, target)
  createCanonicalChanges(state, selected, events, elapsedDays, target)
  introduceAnomalies(state, selected, events, elapsedDays, target)
  generateBankActivity(state, selected, events, elapsedDays, target)
  processLoanRepayments(state, selected, events, target)
  generateProductAdoption(state, selected, events, elapsedDays, target)
  generateTravelActivity(state, selected, events, elapsedDays, target)
  generateCrmActivity(state, selected, events, elapsedDays, target)
  processFlightStatus(state, selected, events, target)

  const arrivalsRandom = randomFor(state.seed, state.tick, "new-people")
  const requestedNewPeople =
    options.newPeople ?? Math.min(100, poisson(arrivalsRandom, state.newPeoplePerDay * elapsedDays))
  if (requestedNewPeople < 0 || requestedNewPeople > 100) {
    throw new IncrementalRequestError("newPeople must be between 0 and 100")
  }
  createNewPeople(state, requestedNewPeople, options.asOf)
  flushSelectedSources(state, selected, events, target)
  if (options.emitCurrentRecord) {
    emitAndCoalesceCurrentRecords(state, selected, events, target)
  }

  enforceEventLimit(events, options.maxEvents)
  store.advanceStream(previousCursor, state, events)
  return {
    state,
    previousCursor,
    events,
    newPeople: requestedNewPeople,
    idempotent: false,
  }
}
