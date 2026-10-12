import { z } from "zod"
import {
  IsoDateSchema,
  IsoDateTimeSchema,
  MoneySchema,
  RegionalNationalIdSchema,
  RegionalPhoneSchema,
} from "./common.js"

export const T24RecordHeaderSchema = z.object({
  id: z.string().min(1),
  status: z.literal("success"),
  transactionStatus: z.enum(["Live", "Unauthorised", "Reversed"]),
  versionNumber: z.string().regex(/^\d+$/),
  recordedAt: IsoDateTimeSchema,
})

export const T24CustomerSchema = z.object({
  application: z.literal("CUSTOMER"),
  header: T24RecordHeaderSchema,
  body: z.object({
    personId: z.string().min(1),
    customerMnemonic: z.string().min(1),
    sectorId: z.number().int().positive(),
    gender: z.enum(["FEMALE", "MALE", "OTHER", "UNSPECIFIED"]),
    nationalityId: z.enum(["VN", "TH"]),
    residenceId: z.enum(["VN", "TH"]),
    displayNames: z.array(z.object({ displayName: z.string().min(1) })).min(1),
    customerNames: z.array(z.object({ customerName: z.string().min(1) })).min(1),
    language: z.enum(["vi-VN", "th-TH"]),
    dateOfBirth: IsoDateSchema,
    communicationDevices: z
      .array(
        z.object({
          phoneNumber: RegionalPhoneSchema,
          email: z.string().email(),
        }),
      )
      .min(1),
    legalIdentifiers: z
      .array(
        z.object({
          legalId: RegionalNationalIdSchema,
          legalDocumentName: z.enum(["CCCD", "THAI_NATIONAL_ID"]),
          countryOfIssue: z.enum(["VN", "TH"]),
          issueDate: IsoDateSchema,
          expiryDate: IsoDateSchema,
        }),
      )
      .min(1),
    addresses: z
      .array(
        z.object({
          addressType: z.enum(["HOME", "MAILING"]),
          addressLine1: z.string().min(1),
          ward: z.string().min(1),
          district: z.string().min(1),
          province: z.string().min(1),
          countryCode: z.enum(["VN", "TH"]),
        }),
      )
      .min(1),
    target: z.number().int().positive(),
    customerStatus: z.number().int().positive(),
    industryId: z.number().int().positive(),
    accountOfficerId: z.string().min(1),
    customerCompany: z.string().min(1),
    amlCheck: z.enum(["NOT_REQUIRED", "PENDING", "PASSED", "FAILED"]),
  }),
})

export const T24AccountSchema = z.object({
  application: z.literal("ACCOUNT"),
  header: T24RecordHeaderSchema,
  body: z.object({
    customerId: z.string().min(1),
    arrangementId: z.string().min(1),
    accountNumber: z.string().min(1),
    accountTitle: z.string().min(1),
    shortTitle: z.string().min(1),
    productLine: z.enum(["ACCOUNTS", "DEPOSITS", "LENDING"]),
    productGroup: z.string().min(1),
    productId: z.string().min(1),
    categoryId: z.number().int().positive(),
    currency: z.enum(["VND", "THB"]),
    companyId: z.string().min(1),
    branchId: z.string().min(1),
    openingDate: IsoDateSchema,
    status: z.enum(["ACTIVE", "DORMANT", "CLOSED", "BLOCKED"]),
    onlineActualBalance: MoneySchema,
    onlineClearedBalance: MoneySchema,
    workingBalance: MoneySchema,
  }),
})

export const T24FundsTransferSchema = z.object({
  application: z.literal("FUNDS.TRANSFER"),
  header: T24RecordHeaderSchema,
  body: z.object({
    transactionReference: z.string().min(1),
    debtorAccount: z.string().min(1),
    creditorAccount: z.string().min(1),
    transactionAmount: MoneySchema,
    transactionType: z.enum([
      "ACCOUNT_TRANSFER",
      "CARD_PAYMENT",
      "LOAN_DISBURSEMENT",
      "LOAN_REPAYMENT",
      "WALLET_PAYMENT",
    ]),
    remittanceInformation: z.string().min(1),
    bookingDate: IsoDateSchema,
    valueDate: IsoDateSchema,
    paymentStatus: z.enum(["PENDING", "COMPLETED", "REJECTED", "REVERSED"]),
  }),
})

export const T24RepaymentScheduleItemSchema = z.object({
  sequence: z.number().int().positive(),
  dueDate: IsoDateSchema,
  principal: MoneySchema,
  interest: MoneySchema,
  status: z.enum(["DUE", "PAID", "OVERDUE", "WAIVED"]),
  paidAt: IsoDateTimeSchema.optional(),
  fundsTransferId: z.string().min(1).optional(),
})

export const T24LoanArrangementSchema = z.object({
  application: z.literal("AA.ARRANGEMENT"),
  header: T24RecordHeaderSchema,
  body: z.object({
    customerId: z.string().min(1),
    arrangementId: z.string().min(1),
    linkedSettlementAccount: z.string().min(1),
    productLine: z.literal("LENDING"),
    productGroup: z.string().min(1),
    productId: z.string().min(1),
    currency: z.enum(["VND", "THB"]),
    originalPrincipal: MoneySchema,
    outstandingPrincipal: MoneySchema,
    annualInterestRate: z.number().min(0).max(1),
    termMonths: z.number().int().positive(),
    repaymentFrequency: z.literal("MONTHLY"),
    startDate: IsoDateSchema,
    maturityDate: IsoDateSchema,
    status: z.enum(["CURRENT", "PAID_OFF", "DELINQUENT", "CLOSED"]),
    repaymentSchedule: z.array(T24RepaymentScheduleItemSchema).min(1),
  }),
})

export type T24Customer = z.infer<typeof T24CustomerSchema>
export type T24Account = z.infer<typeof T24AccountSchema>
export type T24FundsTransfer = z.infer<typeof T24FundsTransferSchema>
export type T24LoanArrangement = z.infer<typeof T24LoanArrangementSchema>
