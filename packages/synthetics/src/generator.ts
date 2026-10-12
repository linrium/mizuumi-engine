import type { SyntheticDataset } from "./schemas/dataset.js"
import { SyntheticDatasetSchema } from "./schemas/dataset.js"
import { getRegionGenerator, type RegionCode } from "./generators/index.js"
import type { Random, RegionGeneratorEngine } from "./generators/types.js"
import { ascii, integer, pick } from "./generators/utils.js"
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
import type { SyntheticPerson } from "./schemas/common.js"

export type GeneratedPersonRecords = {
  person: SyntheticPerson
  vikkiBank: {
    customer: T24Customer
    account: T24Account
    wallet: VikkiWallet
    fundsTransfers: T24FundsTransfer[]
  }
  hdbank: {
    customer: T24Customer
    account: T24Account
    loan: T24LoanArrangement
    fundsTransfers: T24FundsTransfer[]
  }
  vietjetAir: {
    customer: VietjetCustomer
    bookings: VietjetBooking[]
  }
}

export type GeneratedSyntheticData = {
  dataset: SyntheticDataset
  records: GeneratedPersonRecords[]
}

export type GenerateOptions = {
  count: number
  seed: number
  region?: RegionCode
  generatedAt?: Date
}

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
  return { amount, currency: engine.currency }
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
      communicationDevices: [
        { phoneNumber: person.phoneNumber, email: person.emailAddress },
      ],
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
  categoryId: number
  balance: number
  openedAt: Date
  recordedAt: string
}): T24Account {
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
      categoryId: input.categoryId,
      currency: input.engine.currency,
      companyId: input.companyId,
      branchId: input.branchId,
      openingDate: isoDate(input.openedAt),
      status: "ACTIVE",
      onlineActualBalance: money(input.balance, input.engine),
      onlineClearedBalance: money(input.balance, input.engine),
      workingBalance: money(input.balance, input.engine),
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

export function generateSyntheticData(options: GenerateOptions): GeneratedSyntheticData {
  if (!Number.isInteger(options.count) || options.count < 1 || options.count > 1_000) {
    throw new RangeError("count must be an integer between 1 and 1000")
  }
  if (!Number.isInteger(options.seed) || options.seed < 0 || options.seed > 2_147_483_647) {
    throw new RangeError("seed must be an integer between 0 and 2147483647")
  }

  const region = options.region ?? "vn"
  const engine = getRegionGenerator(region)
  const random = mulberry32(options.seed)
  const generatedAtDate = options.generatedAt ?? new Date()
  const generatedAt = generatedAtDate.toISOString()
  const people: SyntheticPerson[] = []
  const records: GeneratedPersonRecords[] = []
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

  for (let index = 0; index < options.count; index += 1) {
    const sequence = index + 1
    const suffix = `${region}-${options.seed}-${String(sequence).padStart(4, "0")}`
    const person = engine.createPerson(random, sequence, options.seed)
    const vikkiCustomerId = `VIK-C-${suffix}`
    const vikkiAccountNumber = `9704${String(options.seed).padStart(10, "0").slice(-10)}${String(sequence).padStart(6, "0")}`
    const vikkiCompanyId = `${engine.countryCode}0010001`
    const vikkiCustomer = createT24Customer(
      person,
      vikkiCustomerId,
      vikkiCompanyId,
      generatedAt,
      engine,
    )
    const vikkiAccount = createAccount({
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
      categoryId: 1001,
      balance: integer(random, ...engine.financials.accountBalance),
      openedAt: addDays(generatedAtDate, -integer(random, 180, 720)),
      recordedAt: generatedAt,
    })
    const wallet: VikkiWallet = {
      walletId: `VIK-W-${suffix}`,
      customerId: vikkiCustomerId,
      settlementAccountNumber: vikkiAccountNumber,
      walletType: "VIKKI_DIGITAL_WALLET",
      activatedAt: vikkiAccount.body.openingDate,
      status: "ACTIVE",
    }

    const walletTransfers: T24FundsTransfer[] = [20, 45, 75].map((daysAgo, transferIndex) =>
      createTransfer({
        id: `VIK-FT-${suffix}-${transferIndex + 1}`,
        reference: `VIK-TXN-${suffix}-${transferIndex + 1}`,
        debtorAccount: vikkiAccountNumber,
        creditorAccount: `MERCHANT-${engine.countryCode}-${transferIndex + 1}`,
        amount: integer(random, ...engine.financials.walletPayment),
        type: "WALLET_PAYMENT",
        description: engine.walletDescription(transferIndex + 1),
        bookedAt: addDays(generatedAtDate, -daysAgo),
        recordedAt: generatedAt,
        engine,
      }),
    )

    const hdbankCustomerId = `HDB-C-${suffix}`
    const hdbankAccountNumber = `9705${String(options.seed).padStart(10, "0").slice(-10)}${String(sequence).padStart(6, "0")}`
    const hdbankCompanyId = `${engine.countryCode}0020001`
    const hdbankCustomer = createT24Customer(
      person,
      hdbankCustomerId,
      hdbankCompanyId,
      generatedAt,
      engine,
    )
    const hdbankAccount = createAccount({
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
      categoryId: 1001,
      balance: integer(random, ...engine.financials.accountBalance),
      openedAt: addDays(generatedAtDate, -integer(random, 365, 1_500)),
      recordedAt: generatedAt,
    })

    const loanStart = addMonths(generatedAtDate, -4)
    const principal = integer(random, 5, 20) * engine.financials.loanUnit
    const monthlyPrincipal = Math.round(principal / 12)
    const loanTransferRecords: T24FundsTransfer[] = []
    const repaymentSchedule = Array.from({ length: 12 }, (_, scheduleIndex) => {
      const installment = scheduleIndex + 1
      const dueDate = addMonths(loanStart, installment)
      const paid = installment <= 3
      const transferId = `HDB-FT-${suffix}-${installment}`
      if (paid) {
        loanTransferRecords.push(
          createTransfer({
            id: transferId,
            reference: `HDB-REPAY-${suffix}-${installment}`,
            debtorAccount: hdbankAccountNumber,
            creditorAccount: `HDB-LOAN-${suffix}`,
            amount: monthlyPrincipal + engine.financials.monthlyInterest,
            type: "LOAN_REPAYMENT",
            description: engine.repaymentDescription(installment),
            bookedAt: dueDate,
            recordedAt: generatedAt,
            engine,
          }),
        )
      }
      return {
        sequence: installment,
        dueDate: isoDate(dueDate),
        principal: money(monthlyPrincipal, engine),
        interest: money(engine.financials.monthlyInterest, engine),
        status: paid ? ("PAID" as const) : ("DUE" as const),
        paidAt: paid ? dueDate.toISOString() : undefined,
        fundsTransferId: paid ? transferId : undefined,
      }
    })
    const loan: T24LoanArrangement = {
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
        outstandingPrincipal: money(principal - monthlyPrincipal * 3, engine),
        annualInterestRate: 0.108,
        termMonths: 12,
        repaymentFrequency: "MONTHLY",
        startDate: isoDate(loanStart),
        maturityDate: isoDate(addMonths(loanStart, 12)),
        status: "CURRENT",
        repaymentSchedule,
      },
    }

    const vietjetCustomer: VietjetCustomer = {
      customerId: `VJ-C-${suffix}`,
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
    const origin = engine.airportForProvince(person.address.province)
    const destinations = engine.destinations.filter((airport) => airport !== origin)
    const bookings: VietjetBooking[] = Array.from({ length: 3 }, (_, bookingIndex) => {
      const number = bookingIndex + 1
      const departure = addDays(generatedAtDate, -(330 - bookingIndex * 120))
      departure.setUTCHours(2 + bookingIndex * 3, 30, 0, 0)
      const arrival = new Date(departure.getTime() + 90 * 60 * 1_000)
      const amount = integer(random, ...engine.financials.booking)
      const usesVikkiWallet = bookingIndex === 0
      const externalTransactionReference = `VIK-VJ-${suffix}`

      return {
        bookingId: `VJ-B-${suffix}-${number}`,
        bookingReference: bookingReference(options.seed, sequence, number),
        customerId: vietjetCustomer.customerId,
        bookedAt: addDays(departure, -21).toISOString(),
        status: "FLOWN",
        passengers: [
          {
            passengerId: `VJ-P-${suffix}`,
            fullName: person.fullName,
            passengerType: "ADULT",
          },
        ],
        segments: [
          {
            flightNumber: `VJ${100 + sequence * 3 + bookingIndex}`,
            departureAirport: origin,
            arrivalAirport: destinations[bookingIndex % destinations.length] ?? "DAD",
            scheduledDepartureAt: departure.toISOString(),
            scheduledArrivalAt: arrival.toISOString(),
            fareClass: "ECO",
            seatNumber: `${integer(random, 1, 30)}${pick(random, ["A", "B", "C", "D", "E", "F"] as const)}`,
          },
        ],
        totalAmount: money(amount, engine),
        payment: {
          paymentId: `VJ-PAY-${suffix}-${number}`,
          method: usesVikkiWallet ? "VIKKI_WALLET" : "BANK_CARD",
          status: "PAID",
          paidAt: addDays(departure, -21).toISOString(),
          externalTransactionReference: usesVikkiWallet
            ? externalTransactionReference
            : undefined,
        },
      }
    })
    const walletBooking = bookings[0] as VietjetBooking
    walletTransfers.push(
      createTransfer({
        id: `VIK-FT-${suffix}-VJ`,
        reference: walletBooking.payment.externalTransactionReference as string,
        debtorAccount: vikkiAccountNumber,
        creditorAccount: "VIETJET-AIR",
        amount: walletBooking.totalAmount.amount,
        type: "WALLET_PAYMENT",
        description: engine.ticketDescription(walletBooking.bookingReference),
        bookedAt: new Date(walletBooking.payment.paidAt as string),
        recordedAt: generatedAt,
        engine,
      }),
    )

    people.push(person)
    vikkiCustomers.push(vikkiCustomer)
    vikkiAccounts.push(vikkiAccount)
    vikkiWallets.push(wallet)
    vikkiTransfers.push(...walletTransfers)
    hdbankCustomers.push(hdbankCustomer)
    hdbankAccounts.push(hdbankAccount)
    hdbankLoans.push(loan)
    hdbankTransfers.push(...loanTransferRecords)
    vietjetCustomers.push(vietjetCustomer)
    vietjetBookings.push(...bookings)
    records.push({
      person,
      vikkiBank: {
        customer: vikkiCustomer,
        account: vikkiAccount,
        wallet,
        fundsTransfers: walletTransfers,
      },
      hdbank: {
        customer: hdbankCustomer,
        account: hdbankAccount,
        loan,
        fundsTransfers: loanTransferRecords,
      },
      vietjetAir: { customer: vietjetCustomer, bookings },
    })
  }

  const dataset = SyntheticDatasetSchema.parse({
    schemaVersion: "1.0.0",
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
        pollBatches: [],
        contacts: [],
        campaignInteractions: [],
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

  return { dataset, records }
}
