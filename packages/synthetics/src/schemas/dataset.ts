import { z } from "zod"
import { IsoDateTimeSchema, SyntheticPersonSchema } from "./common.js"
import { CrmDatasetSchema } from "./organizations/crm.js"
import { HdbankDatasetSchema } from "./organizations/hdbank.js"
import { VietjetAirDatasetSchema } from "./organizations/vietjet-air.js"
import { VikkiBankDatasetSchema } from "./organizations/vikki-bank.js"

function addReferenceIssue(
  context: z.RefinementCtx,
  path: (string | number)[],
  target: string,
) {
  context.addIssue({
    code: z.ZodIssueCode.custom,
    path,
    message: `Referenced ${target} does not exist in this dataset`,
  })
}

function addInvariantIssue(
  context: z.RefinementCtx,
  path: (string | number)[],
  message: string,
) {
  context.addIssue({ code: z.ZodIssueCode.custom, path, message })
}

export const SyntheticDatasetSchema = z
  .object({
    schemaVersion: z.literal("2.0.0"),
    region: z.enum(["vn", "th"]),
    generatedAt: IsoDateTimeSchema,
    seed: z.number().int().nonnegative(),
    people: z.array(SyntheticPersonSchema).min(1),
    organizations: z.object({
      vikkiBank: VikkiBankDatasetSchema,
      hdbank: HdbankDatasetSchema,
      crm: CrmDatasetSchema,
      vietjetAir: VietjetAirDatasetSchema,
    }),
  })
  .superRefine((dataset, context) => {
    const expectedCountry = dataset.region === "vn" ? "VN" : "TH"
    const expectedLanguage = dataset.region === "vn" ? "vi-VN" : "th-TH"
    const expectedTimezone = dataset.region === "vn" ? "Asia/Ho_Chi_Minh" : "Asia/Bangkok"
    const expectedCurrency = dataset.region === "vn" ? "VND" : "THB"
    const people = new Map(dataset.people.map((person) => [person.personId, person]))

    if (people.size !== dataset.people.length) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["people"],
        message: "personId must be unique",
      })
    }

    dataset.people.forEach((person, index) => {
      if (
        person.nationalityCode !== expectedCountry ||
        person.residenceCountryCode !== expectedCountry ||
        person.preferredLanguage !== expectedLanguage
      ) {
        context.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["people", index],
          message: `Person locale does not match dataset region ${dataset.region}`,
        })
      }
    })

    Object.entries(dataset.organizations).forEach(([name, organization]) => {
      if (
        organization.organization.countryCode !== expectedCountry ||
        organization.organization.timezone !== expectedTimezone
      ) {
        context.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["organizations", name, "organization"],
          message: `Organization locale does not match dataset region ${dataset.region}`,
        })
      }
    })

    const vikkiCustomerIds = new Set(
      dataset.organizations.vikkiBank.customers.map((customer) => customer.header.id),
    )
    const vikkiAccountNumbers = new Set(
      dataset.organizations.vikkiBank.accounts.map((account) => account.body.accountNumber),
    )
    const vikkiTransactionReferences = new Set(
      dataset.organizations.vikkiBank.fundsTransfers.map(
        (transfer) => transfer.body.transactionReference,
      ),
    )

    dataset.organizations.vikkiBank.accounts.forEach((account, index) => {
      if (!vikkiCustomerIds.has(account.body.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "vikkiBank", "accounts", index, "body", "customerId"],
          "Vikki CUSTOMER",
        )
      }
      const balances = [
        account.body.onlineActualBalance,
        account.body.onlineClearedBalance,
        account.body.workingBalance,
      ]
      if (
        account.body.currency !== expectedCurrency ||
        balances.some((balance) => balance.currency !== expectedCurrency) ||
        balances.some((balance) => balance.amount !== balances[0]?.amount)
      ) {
        addInvariantIssue(
          context,
          ["organizations", "vikkiBank", "accounts", index, "body"],
          "Vikki account currency and balance snapshots must agree with the dataset region",
        )
      }
    })
    dataset.organizations.vikkiBank.wallets.forEach((wallet, index) => {
      if (!vikkiCustomerIds.has(wallet.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "vikkiBank", "wallets", index, "customerId"],
          "Vikki CUSTOMER",
        )
      }
      if (!vikkiAccountNumbers.has(wallet.settlementAccountNumber)) {
        addReferenceIssue(
          context,
          ["organizations", "vikkiBank", "wallets", index, "settlementAccountNumber"],
          "Vikki ACCOUNT",
        )
      }
      const account = dataset.organizations.vikkiBank.accounts.find(
        (candidate) => candidate.body.accountNumber === wallet.settlementAccountNumber,
      )
      if (account && wallet.activatedAt < account.body.openingDate) {
        addInvariantIssue(
          context,
          ["organizations", "vikkiBank", "wallets", index, "activatedAt"],
          "Wallet cannot be activated before its settlement account opens",
        )
      }
    })
    dataset.organizations.vikkiBank.fundsTransfers.forEach((transfer, index) => {
      if (transfer.body.transactionAmount.currency !== expectedCurrency) {
        addInvariantIssue(
          context,
          ["organizations", "vikkiBank", "fundsTransfers", index, "body", "transactionAmount"],
          "Vikki transfer currency must match the dataset region",
        )
      }
      const account = dataset.organizations.vikkiBank.accounts.find(
        (candidate) => candidate.body.accountNumber === transfer.body.debtorAccount,
      )
      if (account && transfer.body.bookingDate < account.body.openingDate) {
        addInvariantIssue(
          context,
          ["organizations", "vikkiBank", "fundsTransfers", index, "body", "bookingDate"],
          "Transfer cannot be booked before its debtor account opens",
        )
      }
      const wallet = dataset.organizations.vikkiBank.wallets.find(
        (candidate) => candidate.settlementAccountNumber === transfer.body.debtorAccount,
      )
      if (
        wallet &&
        transfer.body.transactionType === "WALLET_PAYMENT" &&
        transfer.body.bookingDate < wallet.activatedAt
      ) {
        addInvariantIssue(
          context,
          ["organizations", "vikkiBank", "fundsTransfers", index, "body", "bookingDate"],
          "Wallet payment cannot be booked before wallet activation",
        )
      }
    })

    const hdbankCustomerIds = new Set(
      dataset.organizations.hdbank.customers.map((customer) => customer.header.id),
    )
    const hdbankAccountNumbers = new Set(
      dataset.organizations.hdbank.accounts.map((account) => account.body.accountNumber),
    )
    const hdbankTransferIds = new Set(
      dataset.organizations.hdbank.fundsTransfers.map((transfer) => transfer.header.id),
    )

    dataset.organizations.hdbank.accounts.forEach((account, index) => {
      if (!hdbankCustomerIds.has(account.body.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "hdbank", "accounts", index, "body", "customerId"],
          "HDBank CUSTOMER",
        )
      }
      const balances = [
        account.body.onlineActualBalance,
        account.body.onlineClearedBalance,
        account.body.workingBalance,
      ]
      if (
        account.body.currency !== expectedCurrency ||
        balances.some((balance) => balance.currency !== expectedCurrency) ||
        balances.some((balance) => balance.amount !== balances[0]?.amount)
      ) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "accounts", index, "body"],
          "HDBank account currency and balance snapshots must agree with the dataset region",
        )
      }
    })
    dataset.organizations.hdbank.fundsTransfers.forEach((transfer, index) => {
      if (transfer.body.transactionAmount.currency !== expectedCurrency) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "fundsTransfers", index, "body", "transactionAmount"],
          "HDBank transfer currency must match the dataset region",
        )
      }
      const account = dataset.organizations.hdbank.accounts.find(
        (candidate) => candidate.body.accountNumber === transfer.body.debtorAccount,
      )
      if (account && transfer.body.bookingDate < account.body.openingDate) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "fundsTransfers", index, "body", "bookingDate"],
          "Transfer cannot be booked before its debtor account opens",
        )
      }
    })
    dataset.organizations.hdbank.loans.forEach((loan, loanIndex) => {
      if (!hdbankCustomerIds.has(loan.body.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "hdbank", "loans", loanIndex, "body", "customerId"],
          "HDBank CUSTOMER",
        )
      }
      if (!hdbankAccountNumbers.has(loan.body.linkedSettlementAccount)) {
        addReferenceIssue(
          context,
          [
            "organizations",
            "hdbank",
            "loans",
            loanIndex,
            "body",
            "linkedSettlementAccount",
          ],
          "HDBank ACCOUNT",
        )
      }
      const linkedAccount = dataset.organizations.hdbank.accounts.find(
        (account) => account.body.accountNumber === loan.body.linkedSettlementAccount,
      )
      if (linkedAccount && loan.body.startDate < linkedAccount.body.openingDate) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "loans", loanIndex, "body", "startDate"],
          "Loan cannot start before its settlement account opens",
        )
      }
      const scheduledPrincipal = loan.body.repaymentSchedule.reduce(
        (sum, item) => sum + item.principal.amount,
        0,
      )
      const paidPrincipal = loan.body.repaymentSchedule
        .filter((item) => item.status === "PAID")
        .reduce((sum, item) => sum + item.principal.amount, 0)
      if (
        loan.body.currency !== expectedCurrency ||
        loan.body.originalPrincipal.currency !== expectedCurrency ||
        loan.body.outstandingPrincipal.currency !== expectedCurrency ||
        loan.body.repaymentSchedule.some(
          (item) =>
            item.principal.currency !== expectedCurrency ||
            item.interest.currency !== expectedCurrency,
        )
      ) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "loans", loanIndex, "body"],
          "Loan currencies must match the dataset region",
        )
      }
      if (
        scheduledPrincipal !== loan.body.originalPrincipal.amount ||
        loan.body.outstandingPrincipal.amount !==
          loan.body.originalPrincipal.amount - paidPrincipal
      ) {
        addInvariantIssue(
          context,
          ["organizations", "hdbank", "loans", loanIndex, "body", "repaymentSchedule"],
          "Loan principal schedule and outstanding amount do not reconcile",
        )
      }
      loan.body.repaymentSchedule.forEach((item, scheduleIndex) => {
        if (item.fundsTransferId && !hdbankTransferIds.has(item.fundsTransferId)) {
          addReferenceIssue(
            context,
            [
              "organizations",
              "hdbank",
              "loans",
              loanIndex,
              "body",
              "repaymentSchedule",
              scheduleIndex,
              "fundsTransferId",
            ],
            "HDBank FUNDS.TRANSFER",
          )
        }
      })
    })

    const crmContactIds = new Set(
      dataset.organizations.crm.contacts.map((contact) => contact.contactId),
    )
    dataset.organizations.crm.campaignInteractions.forEach((interaction, index) => {
      if (!crmContactIds.has(interaction.contactId)) {
        addReferenceIssue(
          context,
          ["organizations", "crm", "campaignInteractions", index, "contactId"],
          "CRM contact",
        )
      }
    })

    const vietjetCustomerIds = new Set(
      dataset.organizations.vietjetAir.customers.map((customer) => customer.customerId),
    )
    const hdbankTransactionReferences = new Map(
      dataset.organizations.hdbank.fundsTransfers.map((transfer) => [
        transfer.body.transactionReference,
        transfer,
      ]),
    )
    dataset.organizations.vietjetAir.bookings.forEach((booking, index) => {
      if (!vietjetCustomerIds.has(booking.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "vietjetAir", "bookings", index, "customerId"],
          "VietJet customer",
        )
      }
      if (booking.totalAmount.currency !== expectedCurrency) {
        addInvariantIssue(
          context,
          ["organizations", "vietjetAir", "bookings", index, "totalAmount"],
          "Booking currency must match the dataset region",
        )
      }
      if (
        booking.payment.method === "VIKKI_WALLET" &&
        dataset.organizations.vikkiBank.customers.length > 0 &&
        (!booking.payment.externalTransactionReference ||
          !vikkiTransactionReferences.has(booking.payment.externalTransactionReference))
      ) {
        addReferenceIssue(
          context,
          [
            "organizations",
            "vietjetAir",
            "bookings",
            index,
            "payment",
            "externalTransactionReference",
          ],
          "Vikki FUNDS.TRANSFER transactionReference",
        )
      }
      const linkedTransfer = booking.payment.externalTransactionReference
        ? booking.payment.method === "VIKKI_WALLET"
          ? dataset.organizations.vikkiBank.fundsTransfers.find(
              (transfer) =>
                transfer.body.transactionReference === booking.payment.externalTransactionReference,
            )
          : hdbankTransactionReferences.get(booking.payment.externalTransactionReference)
        : undefined
      const linkedSourceIncluded =
        booking.payment.method === "VIKKI_WALLET"
          ? dataset.organizations.vikkiBank.customers.length > 0
          : booking.payment.method === "BANK_CARD"
            ? dataset.organizations.hdbank.customers.length > 0
            : false
      if (
        linkedSourceIncluded &&
        (!linkedTransfer ||
          linkedTransfer.body.transactionAmount.amount !== booking.totalAmount.amount ||
          linkedTransfer.body.transactionAmount.currency !== booking.totalAmount.currency)
      ) {
        addReferenceIssue(
          context,
          ["organizations", "vietjetAir", "bookings", index, "payment"],
          "matching bank payment amount",
        )
      }
      booking.segments.forEach((segment, segmentIndex) => {
        if (
          booking.bookedAt > segment.scheduledDepartureAt ||
          segment.scheduledDepartureAt >= segment.scheduledArrivalAt ||
          (booking.payment.paidAt && booking.payment.paidAt < booking.bookedAt) ||
          (booking.payment.paidAt && booking.payment.paidAt > segment.scheduledDepartureAt)
        ) {
          addInvariantIssue(
            context,
            ["organizations", "vietjetAir", "bookings", index, "segments", segmentIndex],
            "Booking, payment, and flight timestamps are not chronological",
          )
        }
      })
    })
  })

export type SyntheticDataset = z.infer<typeof SyntheticDatasetSchema>
