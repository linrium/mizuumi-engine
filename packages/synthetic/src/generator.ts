import { createWriteStream } from "node:fs"
import { mkdir, readdir, rm, writeFile } from "node:fs/promises"
import { once } from "node:events"
import path from "node:path"

export interface GenerateOptions {
  outputDirectory: string
  customerCount: number
  seed: number
  asOf: string
  force: boolean
}

interface Person {
  ordinal: number
  givenName: string
  familyName: string
  dateOfBirth: string
  gender: "F" | "M"
  nationalId: string
  phone: string
  email: string
  address: string
  city: string
  provinceCode: string
}

type Row = Record<string, string | number | boolean>

const GIVEN_NAMES = [
  "An", "Anh", "Bảo", "Châu", "Chi", "Dũng", "Giang", "Hà", "Hải", "Hiếu",
  "Hoàng", "Hùng", "Huy", "Khánh", "Lan", "Linh", "Long", "Mai", "Minh", "Nam",
  "Ngân", "Ngọc", "Nhung", "Phúc", "Phương", "Quân", "Quang", "Thảo", "Trang", "Tuấn",
  "Uyên", "Việt", "Yến",
]

const FAMILY_NAMES = [
  "Nguyễn", "Trần", "Lê", "Phạm", "Hoàng", "Huỳnh", "Phan", "Vũ", "Võ", "Đặng",
  "Bùi", "Đỗ", "Hồ", "Ngô", "Dương", "Lý",
]

const LOCATIONS = [
  { city: "Hồ Chí Minh", provinceCode: "079", weight: 28, streets: ["Nguyễn Huệ", "Lê Lợi", "Điện Biên Phủ", "Phan Xích Long"] },
  { city: "Hà Nội", provinceCode: "001", weight: 22, streets: ["Lý Thường Kiệt", "Trần Duy Hưng", "Kim Mã", "Nguyễn Trãi"] },
  { city: "Đà Nẵng", provinceCode: "048", weight: 9, streets: ["Bạch Đằng", "Nguyễn Văn Linh", "Võ Nguyên Giáp"] },
  { city: "Hải Phòng", provinceCode: "031", weight: 7, streets: ["Lạch Tray", "Tô Hiệu", "Cầu Đất"] },
  { city: "Cần Thơ", provinceCode: "092", weight: 7, streets: ["30 Tháng 4", "Nguyễn Văn Cừ", "Mậu Thân"] },
  { city: "Bình Dương", provinceCode: "074", weight: 7, streets: ["Yersin", "Cách Mạng Tháng Tám", "Lê Hồng Phong"] },
  { city: "Đồng Nai", provinceCode: "075", weight: 6, streets: ["Phạm Văn Thuận", "Võ Thị Sáu", "Đồng Khởi"] },
  { city: "Khánh Hòa", provinceCode: "056", weight: 5, streets: ["Trần Phú", "Lê Thánh Tôn", "Thái Nguyên"] },
  { city: "Thừa Thiên Huế", provinceCode: "046", weight: 4, streets: ["Hùng Vương", "Lê Lợi", "Điện Biên Phủ"] },
  { city: "Quảng Ninh", provinceCode: "022", weight: 5, streets: ["Trần Quốc Nghiễn", "Hạ Long", "Lê Thánh Tông"] },
]

const ROUTES = [
  ["SGN", "HAN"], ["SGN", "DAD"], ["SGN", "CXR"], ["SGN", "PQC"],
  ["HAN", "DAD"], ["HAN", "CXR"], ["HAN", "PQC"], ["DAD", "SGN"],
]

const TRANSACTION_TEMPLATES = [
  ["food_and_beverage", "Highlands Coffee", 55_000, 140_000],
  ["groceries", "WinMart", 180_000, 1_250_000],
  ["transport", "Grab", 35_000, 280_000],
  ["utilities", "EVN", 420_000, 1_900_000],
  ["telecom", "Viettel", 100_000, 500_000],
  ["ecommerce", "Shopee", 120_000, 2_400_000],
  ["dining", "Pizza 4P's", 350_000, 1_600_000],
] as const

const FILE_HEADERS = {
  "vikki/customers.csv": ["customer_id", "full_name", "date_of_birth", "national_id", "phone", "email", "address", "city", "kyc_status", "created_at"],
  "vikki/accounts.csv": ["account_id", "customer_id", "account_type", "currency", "balance", "wallet_is_default", "opened_at", "status"],
  "vikki/wallet_transactions.csv": ["transaction_id", "account_id", "occurred_at", "direction", "category", "merchant", "amount_vnd", "status", "reference"],
  "hdbank/customers.csv": ["customer_id", "full_name", "date_of_birth", "national_id", "phone", "email", "address", "city", "segment", "kyc_status", "created_at"],
  "hdbank/loans.csv": ["loan_id", "customer_id", "product", "principal_vnd", "annual_rate_pct", "term_months", "outstanding_vnd", "originated_at", "status"],
  "hdbank/loan_repayments.csv": ["repayment_id", "loan_id", "installment_number", "due_date", "paid_at", "principal_vnd", "interest_vnd", "status"],
  "crm/contacts.csv": ["contact_id", "full_name", "date_of_birth", "national_id", "phone", "email", "city", "source", "created_at", "updated_at"],
  "crm/campaign_interactions.csv": ["interaction_id", "contact_id", "campaign_id", "channel", "occurred_at", "outcome", "consent_status"],
  "vietjet/passengers.csv": ["passenger_id", "full_name", "date_of_birth", "national_id", "phone", "email", "loyalty_tier", "created_at"],
  "vietjet/bookings.csv": ["booking_id", "passenger_id", "pnr", "booked_at", "departure_at", "origin", "destination", "fare_class", "fare_vnd", "payment_method", "payment_reference", "status"],
} as const

class Random {
  constructor(private state: number) {}

  next(): number {
    this.state |= 0
    this.state = (this.state + 0x6d2b79f5) | 0
    let value = Math.imul(this.state ^ (this.state >>> 15), 1 | this.state)
    value = value + Math.imul(value ^ (value >>> 7), 61 | value) ^ value
    return ((value ^ (value >>> 14)) >>> 0) / 4_294_967_296
  }

  int(min: number, max: number): number {
    return min + Math.floor(this.next() * (max - min + 1))
  }

  pick<T>(values: readonly T[]): T {
    return values[this.int(0, values.length - 1)]
  }

  chance(probability: number): boolean {
    return this.next() < probability
  }
}

class CsvSink {
  private readonly stream
  count = 0

  constructor(readonly filePath: string, readonly headers: readonly string[]) {
    this.stream = createWriteStream(filePath, { encoding: "utf8" })
    this.stream.write(`${headers.join(",")}\n`)
  }

  async write(row: Row): Promise<void> {
    const line = `${this.headers.map((header) => csvValue(row[header] ?? "")).join(",")}\n`
    this.count += 1
    if (!this.stream.write(line)) await once(this.stream, "drain")
  }

  async close(): Promise<void> {
    this.stream.end()
    await once(this.stream, "finish")
  }
}

function csvValue(value: Row[string]): string {
  const text = String(value)
  return /[",\n\r]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

function ascii(value: string): string {
  return value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").replaceAll("đ", "d").replaceAll("Đ", "D")
}

function isoDate(date: Date): string {
  return date.toISOString().slice(0, 10)
}

function isoTimestamp(date: Date): string {
  return date.toISOString().replace(".000Z", "+07:00")
}

function daysBefore(asOf: Date, days: number, hour = 3): Date {
  return new Date(asOf.getTime() - days * 86_400_000 + hour * 3_600_000)
}

function weightedLocation(rng: Random): (typeof LOCATIONS)[number] {
  let value = rng.int(1, LOCATIONS.reduce((sum, item) => sum + item.weight, 0))
  for (const location of LOCATIONS) {
    value -= location.weight
    if (value <= 0) return location
  }
  return LOCATIONS[0]
}

function makeNationalId(person: Pick<Person, "provinceCode" | "gender" | "dateOfBirth" | "ordinal">): string {
  const birthYear = Number(person.dateOfBirth.slice(0, 4))
  const centuryDigit = birthYear >= 2000 ? (person.gender === "F" ? "3" : "2") : (person.gender === "F" ? "1" : "0")
  return `${person.provinceCode}${centuryDigit}${String(birthYear).slice(2)}${String(700_000 + person.ordinal).slice(-6)}`
}

function makePerson(ordinal: number, rng: Random): Person {
  const givenName = rng.pick(GIVEN_NAMES)
  const familyName = rng.pick(FAMILY_NAMES)
  const middleName = rng.chance(0.72) ? rng.pick(["Văn", "Thị", "Minh", "Ngọc", "Thanh", "Hoài"]) : ""
  const location = weightedLocation(rng)
  const gender = rng.chance(0.51) ? "F" : "M"
  const year = rng.int(1958, 2005)
  const month = rng.int(1, 12)
  const day = rng.int(1, 28)
  const nameSlug = ascii(`${givenName}.${familyName}`).toLowerCase().replaceAll(" ", "")
  const subscriber = String(1_000_000 + ordinal).slice(-7)
  const person: Person = {
    ordinal,
    givenName,
    familyName: [familyName, middleName].filter(Boolean).join(" "),
    dateOfBirth: `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`,
    gender,
    nationalId: "",
    phone: `+84${rng.pick(["32", "35", "38", "39", "70", "76", "77", "81", "90", "93"])}${subscriber}`,
    email: `${nameSlug}.${String(ordinal).padStart(6, "0")}@${rng.pick(["gmail.com", "outlook.com", "yahoo.com", "icloud.com"])}`,
    address: `${rng.int(1, 420)} ${rng.pick(location.streets)}, ${location.city}`,
    city: location.city,
    provinceCode: location.provinceCode,
  }
  person.nationalId = makeNationalId(person)
  return person
}

function fullName(person: Person): string {
  return `${person.familyName} ${person.givenName}`
}

function noisyIdentity(person: Person, rng: Random, source: "crm" | "other"): Pick<Person, "nationalId" | "phone" | "email"> {
  if (source === "crm") {
    return {
      nationalId: rng.chance(0.52) ? person.nationalId : "",
      phone: rng.chance(0.2) ? driftPhone(person.phone) : person.phone,
      email: rng.chance(0.12) ? person.email.replace("@", "+old@") : person.email,
    }
  }
  return {
    nationalId: rng.chance(0.08) ? "" : person.nationalId,
    phone: rng.chance(0.025) ? driftPhone(person.phone) : person.phone,
    email: rng.chance(0.02) ? person.email.replace("@", "+bank@") : person.email,
  }
}

function driftPhone(phone: string): string {
  const lastDigit = Number(phone.at(-1) ?? "0")
  return `${phone.slice(0, -1)}${(lastDigit + 3) % 10}`
}

async function prepareOutput(options: GenerateOptions): Promise<void> {
  const outputDirectory = path.resolve(options.outputDirectory)
  await mkdir(outputDirectory, { recursive: true })
  const entries = await readdir(outputDirectory)
  if (entries.length > 0 && !options.force) {
    throw new Error(`Output directory is not empty: ${outputDirectory}. Use --force to replace it.`)
  }
  if (entries.length > 0) {
    await Promise.all(entries.map((entry) => rm(path.join(outputDirectory, entry), { recursive: true, force: true })))
  }
  await Promise.all(["vikki", "hdbank", "crm", "vietjet"].map((source) => mkdir(path.join(outputDirectory, source), { recursive: true })))
}

function sourceDates(asOf: Date, rng: Random): { createdAt: string; openedAt: string } {
  const createdDaysAgo = rng.int(180, 2_400)
  return {
    createdAt: isoTimestamp(daysBefore(asOf, createdDaysAgo)),
    openedAt: isoDate(daysBefore(asOf, createdDaysAgo - rng.int(0, 60))),
  }
}

async function writeVikki(sinks: Record<string, CsvSink>, person: Person, rng: Random, asOf: Date, featured: boolean): Promise<void> {
  const customerId = featured ? "VIK-CUST-000001" : `VIK-CUST-${String(person.ordinal).padStart(6, "0")}`
  const accountId = featured ? "VIK-ACC-000001" : `VIK-ACC-${String(person.ordinal).padStart(6, "0")}`
  const dates = sourceDates(asOf, rng)
  const identity = featured ? person : noisyIdentity(person, rng, "other")
  await sinks["vikki/customers.csv"].write({
    customer_id: customerId, full_name: fullName(person), date_of_birth: person.dateOfBirth,
    national_id: identity.nationalId, phone: identity.phone, email: identity.email,
    address: person.address, city: person.city, kyc_status: "verified", created_at: dates.createdAt,
  })
  await sinks["vikki/accounts.csv"].write({
    account_id: accountId, customer_id: customerId, account_type: "digital_transaction",
    currency: "VND", balance: featured ? 18_760_000 : rng.int(250_000, 95_000_000),
    wallet_is_default: featured || rng.chance(0.73), opened_at: dates.openedAt, status: "active",
  })
  const transactionCount = featured ? 18 : rng.int(3, 12)
  for (let index = 1; index <= transactionCount; index += 1) {
    const template = rng.pick(TRANSACTION_TEMPLATES)
    const daysAgo = featured ? Math.round(2 + (index - 1) * 86 / (transactionCount - 1)) : rng.int(1, 90)
    await sinks["vikki/wallet_transactions.csv"].write({
      transaction_id: `${featured ? "VIK-TXN-MA" : `VIK-TXN-${String(person.ordinal).padStart(6, "0")}`}-${String(index).padStart(2, "0")}`,
      account_id: accountId, occurred_at: isoTimestamp(daysBefore(asOf, daysAgo, rng.int(1, 15))),
      direction: "debit", category: template[0], merchant: template[1], amount_vnd: rng.int(template[2] / 1_000, template[3] / 1_000) * 1_000,
      status: "completed", reference: "",
    })
  }
  if (featured) {
    await sinks["vikki/wallet_transactions.csv"].write({
      transaction_id: "VIK-TXN-VJ-8K2M4Q", account_id: accountId,
      occurred_at: isoTimestamp(daysBefore(asOf, 38, 4)), direction: "debit", category: "travel",
      merchant: "VietJet Air", amount_vnd: 2_186_000, status: "completed", reference: "VJ-8K2M4Q",
    })
  }
}

async function writeHdbank(sinks: Record<string, CsvSink>, person: Person, rng: Random, asOf: Date, featured: boolean): Promise<void> {
  const customerId = featured ? "HDB-CUST-000001" : `HDB-CUST-${String(person.ordinal).padStart(6, "0")}`
  const dates = sourceDates(asOf, rng)
  const identity = featured ? person : noisyIdentity(person, rng, "other")
  await sinks["hdbank/customers.csv"].write({
    customer_id: customerId, full_name: fullName(person), date_of_birth: person.dateOfBirth,
    national_id: identity.nationalId, phone: identity.phone, email: identity.email,
    address: person.address, city: person.city, segment: featured ? "mass_affluent" : rng.pick(["mass", "mass", "mass_affluent", "young_professional"]),
    kyc_status: "verified", created_at: dates.createdAt,
  })
  if (!featured && !rng.chance(0.24)) return
  const loanId = featured ? "HDB-LOAN-000001" : `HDB-LOAN-${String(person.ordinal).padStart(6, "0")}`
  const principal = featured ? 180_000_000 : rng.int(30, 500) * 1_000_000
  const rate = featured ? 12.4 : rng.int(88, 168) / 10
  const originatedDaysAgo = featured ? 112 : rng.int(40, 1_000)
  const repayments = featured ? 3 : Math.min(rng.int(0, 9), Math.floor(originatedDaysAgo / 30))
  const monthlyPrincipal = Math.round(principal / (featured ? 36 : 24) / 1_000) * 1_000
  await sinks["hdbank/loans.csv"].write({
    loan_id: loanId, customer_id: customerId, product: "unsecured_consumer_loan", principal_vnd: principal,
    annual_rate_pct: rate.toFixed(1), term_months: featured ? 36 : 24,
    outstanding_vnd: principal - repayments * monthlyPrincipal, originated_at: isoDate(daysBefore(asOf, originatedDaysAgo)), status: "current",
  })
  for (let index = 1; index <= repayments; index += 1) {
    const dueDaysAgo = originatedDaysAgo - index * 30
    await sinks["hdbank/loan_repayments.csv"].write({
      repayment_id: `${loanId}-PMT-${String(index).padStart(2, "0")}`, loan_id: loanId, installment_number: index,
      due_date: isoDate(daysBefore(asOf, dueDaysAgo)), paid_at: isoTimestamp(daysBefore(asOf, dueDaysAgo - rng.int(0, 2), 2)),
      principal_vnd: monthlyPrincipal, interest_vnd: Math.round((principal - (index - 1) * monthlyPrincipal) * rate / 1_200 / 1_000) * 1_000,
      status: "paid",
    })
  }
}

async function writeCrm(sinks: Record<string, CsvSink>, person: Person, rng: Random, asOf: Date, featured: boolean): Promise<void> {
  const contactId = featured ? "CRM-CONTACT-000001" : `CRM-CONTACT-${String(person.ordinal).padStart(6, "0")}`
  const identity = featured ? { nationalId: "", phone: "+84912204567", email: person.email } : noisyIdentity(person, rng, "crm")
  const createdDaysAgo = featured ? 720 : rng.int(60, 1_800)
  await sinks["crm/contacts.csv"].write({
    contact_id: contactId, full_name: featured ? "Tran Minh Anh" : (rng.chance(0.35) ? ascii(fullName(person)) : fullName(person)),
    date_of_birth: person.dateOfBirth, national_id: identity.nationalId, phone: identity.phone,
    email: identity.email, city: person.city, source: featured ? "legacy_campaign_import" : rng.pick(["web_form", "branch_referral", "campaign_import"]),
    created_at: isoTimestamp(daysBefore(asOf, createdDaysAgo)), updated_at: isoTimestamp(daysBefore(asOf, featured ? 210 : rng.int(1, createdDaysAgo))),
  })
  if (featured || rng.chance(0.58)) {
    await sinks["crm/campaign_interactions.csv"].write({
      interaction_id: featured ? "CRM-INT-000001" : `CRM-INT-${String(person.ordinal).padStart(6, "0")}`,
      contact_id: contactId, campaign_id: featured ? "CMP-TRAVEL-2026-Q3" : rng.pick(["CMP-CARD-2026-Q2", "CMP-SAVINGS-2026-Q3", "CMP-TRAVEL-2026-Q3"]),
      channel: featured ? "sms" : rng.pick(["email", "sms", "phone"]), occurred_at: isoTimestamp(daysBefore(asOf, featured ? 62 : rng.int(2, 180), 3)),
      outcome: featured ? "clicked" : rng.pick(["delivered", "opened", "clicked", "no_answer"]), consent_status: "opted_in",
    })
  }
}

async function writeVietjet(sinks: Record<string, CsvSink>, person: Person, rng: Random, asOf: Date, featured: boolean): Promise<void> {
  const passengerId = featured ? "VJ-PAX-000001" : `VJ-PAX-${String(person.ordinal).padStart(6, "0")}`
  const identity = featured ? person : noisyIdentity(person, rng, "other")
  await sinks["vietjet/passengers.csv"].write({
    passenger_id: passengerId, full_name: fullName(person), date_of_birth: person.dateOfBirth,
    national_id: identity.nationalId, phone: identity.phone, email: identity.email,
    loyalty_tier: featured ? "member" : rng.pick(["none", "none", "member", "silver"]), created_at: isoTimestamp(daysBefore(asOf, featured ? 890 : rng.int(20, 2_000))),
  })
  const bookingCount = featured ? 3 : rng.int(1, 4)
  const featuredDays = [310, 146, 38]
  for (let index = 1; index <= bookingCount; index += 1) {
    const route = featured ? (["SGN", ["HAN", "DAD", "CXR"][index - 1]] as const) : rng.pick(ROUTES)
    const daysAgo = featured ? featuredDays[index - 1] : rng.int(3, 364)
    const bookingId = featured && index === 3 ? "VJ-8K2M4Q" : `VJ-BKG-${String(person.ordinal).padStart(6, "0")}-${index}`
    const paidWithVikki = featured && index === 3
    await sinks["vietjet/bookings.csv"].write({
      booking_id: bookingId, passenger_id: passengerId, pnr: featured && index === 3 ? "8K2M4Q" : `${String(person.ordinal).slice(-4).padStart(4, "0")}${String.fromCharCode(64 + index)}Q`,
      booked_at: isoTimestamp(daysBefore(asOf, daysAgo, 4)), departure_at: isoTimestamp(daysBefore(asOf, daysAgo - rng.int(7, 28), 1)),
      origin: route[0], destination: route[1], fare_class: rng.pick(["eco", "eco", "deluxe"]),
      fare_vnd: paidWithVikki ? 2_186_000 : rng.int(850, 3_800) * 1_000,
      payment_method: paidWithVikki ? "vikki_wallet" : rng.pick(["domestic_card", "credit_card", "bank_transfer"]),
      payment_reference: paidWithVikki ? "VIK-TXN-VJ-8K2M4Q" : "", status: "flown",
    })
  }
}

function featuredPerson(): Person {
  return {
    ordinal: 1,
    givenName: "Anh",
    familyName: "Trần Minh",
    dateOfBirth: "1992-04-18",
    gender: "F",
    nationalId: "079192700001",
    phone: "+84902204567",
    email: "minh.anh.tran@example.vn",
    address: "128 Nguyễn Đình Chiểu, Quận 3, Hồ Chí Minh",
    city: "Hồ Chí Minh",
    provinceCode: "079",
  }
}

export async function generateDataset(options: GenerateOptions): Promise<Record<string, number>> {
  if (!Number.isSafeInteger(options.customerCount) || options.customerCount < 1) throw new Error("customerCount must be a positive integer")
  if (!Number.isSafeInteger(options.seed)) throw new Error("seed must be an integer")
  if (!/^\d{4}-\d{2}-\d{2}$/.test(options.asOf)) throw new Error("asOf must use YYYY-MM-DD format")
  const asOf = new Date(`${options.asOf}T00:00:00.000Z`)
  if (Number.isNaN(asOf.getTime()) || isoDate(asOf) !== options.asOf) throw new Error("asOf must be a valid calendar date")

  await prepareOutput(options)
  const outputDirectory = path.resolve(options.outputDirectory)
  const sinks: Record<string, CsvSink> = {}
  for (const [relativePath, headers] of Object.entries(FILE_HEADERS)) {
    sinks[relativePath] = new CsvSink(path.join(outputDirectory, relativePath), headers)
  }

  const rng = new Random(options.seed)
  for (let ordinal = 1; ordinal <= options.customerCount; ordinal += 1) {
    const featured = ordinal === 1
    const person = featured ? featuredPerson() : makePerson(ordinal, rng)
    let inVikki = featured || rng.chance(0.68)
    let inHdbank = featured || rng.chance(0.82)
    let inCrm = featured || rng.chance(0.55)
    let inVietjet = featured || rng.chance(0.42)
    if (!inVikki && !inHdbank && !inCrm && !inVietjet) inHdbank = true
    if (inVikki) await writeVikki(sinks, person, rng, asOf, featured)
    if (inHdbank) await writeHdbank(sinks, person, rng, asOf, featured)
    if (inCrm) await writeCrm(sinks, person, rng, asOf, featured)
    if (inVietjet) await writeVietjet(sinks, person, rng, asOf, featured)
  }

  await Promise.all(Object.values(sinks).map((sink) => sink.close()))
  const rowCounts = Object.fromEntries(Object.entries(sinks).map(([name, sink]) => [name, sink.count]))
  const manifest = {
    schema_version: "1.0",
    generated_at: new Date().toISOString(),
    as_of_date: options.asOf,
    seed: options.seed,
    synthetic_people: options.customerCount,
    featured_customer: {
      name: "Trần Minh Anh",
      vikki_customer_id: "VIK-CUST-000001",
      hdbank_customer_id: "HDB-CUST-000001",
      crm_contact_id: "CRM-CONTACT-000001",
      vietjet_passenger_id: "VJ-PAX-000001",
    },
    row_counts: rowCounts,
    notice: "Entirely synthetic demo data. No record represents a real person.",
  }
  await writeFile(path.join(outputDirectory, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`, "utf8")
  return rowCounts
}
