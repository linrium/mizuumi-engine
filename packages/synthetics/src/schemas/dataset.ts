import { z } from "zod"
import { IsoDateTimeSchema, SyntheticPersonSchema, type SyntheticPerson } from "./common.js"
import { CrmDatasetSchema } from "./organizations/crm.js"
import { HdbankDatasetSchema } from "./organizations/hdbank.js"
import { VietjetAirDatasetSchema } from "./organizations/vietjet-air.js"
import { VikkiBankDatasetSchema } from "./organizations/vikki-bank.js"
import type { T24Customer } from "./temenos-t24.js"

function addIdentityIssue(
  context: z.RefinementCtx,
  path: (string | number)[],
  field: string,
) {
  context.addIssue({
    code: z.ZodIssueCode.custom,
    path,
    message: `${field} does not match the canonical Vietnamese person`,
  })
}

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

function validateT24Customer(
  customer: T24Customer,
  person: SyntheticPerson,
  context: z.RefinementCtx,
  path: (string | number)[],
) {
  const body = customer.body
  const communication = body.communicationDevices[0]
  const legalIdentifier = body.legalIdentifiers[0]

  if (body.displayNames[0]?.displayName !== person.fullName) {
    addIdentityIssue(context, [...path, "body", "displayNames", 0], "fullName")
  }
  if (body.customerNames[0]?.customerName !== person.fullName) {
    addIdentityIssue(context, [...path, "body", "customerNames", 0], "fullName")
  }
  if (body.dateOfBirth !== person.dateOfBirth) {
    addIdentityIssue(context, [...path, "body", "dateOfBirth"], "dateOfBirth")
  }
  if (body.gender !== person.gender) {
    addIdentityIssue(context, [...path, "body", "gender"], "gender")
  }
  if (communication?.phoneNumber !== person.phoneNumber) {
    addIdentityIssue(context, [...path, "body", "communicationDevices", 0, "phoneNumber"], "phoneNumber")
  }
  if (communication?.email !== person.emailAddress) {
    addIdentityIssue(context, [...path, "body", "communicationDevices", 0, "email"], "emailAddress")
  }
  if (legalIdentifier?.legalId !== person.nationalId) {
    addIdentityIssue(context, [...path, "body", "legalIdentifiers", 0, "legalId"], "nationalId")
  }
}

export const SyntheticDatasetSchema = z
  .object({
    schemaVersion: z.literal("1.0.0"),
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

    const bankCustomers = [
      ["vikkiBank", dataset.organizations.vikkiBank.customers] as const,
      ["hdbank", dataset.organizations.hdbank.customers] as const,
    ]

    for (const [bankName, customers] of bankCustomers) {
      customers.forEach((customer, index) => {
        const person = people.get(customer.body.personId)
        const path = ["organizations", bankName, "customers", index]
        if (!person) {
          addIdentityIssue(context, [...path, "body", "personId"], "personId")
          return
        }
        validateT24Customer(customer, person, context, path)
      })
    }

    dataset.organizations.vietjetAir.customers.forEach((customer, index) => {
      const person = people.get(customer.personId)
      const path = ["organizations", "vietjetAir", "customers", index]
      if (!person) {
        addIdentityIssue(context, [...path, "personId"], "personId")
        return
      }

      const comparisons = {
        fullName: person.fullName,
        dateOfBirth: person.dateOfBirth,
        gender: person.gender,
        nationalId: person.nationalId,
        phoneNumber: person.phoneNumber,
        emailAddress: person.emailAddress,
      } as const

      for (const [field, expected] of Object.entries(comparisons)) {
        if (customer[field as keyof typeof comparisons] !== expected) {
          addIdentityIssue(context, [...path, field], field)
        }
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
    dataset.organizations.crm.contacts.forEach((contact, index) => {
      if (!people.has(contact.personId)) {
        addReferenceIssue(
          context,
          ["organizations", "crm", "contacts", index, "personId"],
          "canonical person",
        )
      }
    })
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
    dataset.organizations.vietjetAir.bookings.forEach((booking, index) => {
      if (!vietjetCustomerIds.has(booking.customerId)) {
        addReferenceIssue(
          context,
          ["organizations", "vietjetAir", "bookings", index, "customerId"],
          "VietJet customer",
        )
      }
      if (
        booking.payment.method === "VIKKI_WALLET" &&
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
    })
  })

export type SyntheticDataset = z.infer<typeof SyntheticDatasetSchema>
