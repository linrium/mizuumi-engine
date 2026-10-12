import type { SyntheticPerson } from "../schemas/common.js"

export type RegionCode = "vn" | "th"
export type CountryCode = "VN" | "TH"
export type LocaleCode = "vi-VN" | "th-TH"
export type CurrencyCode = "VND" | "THB"
export type RegionTimezone = "Asia/Ho_Chi_Minh" | "Asia/Bangkok"
export type Random = () => number

export type RegionGeneratorEngine = {
  region: RegionCode
  countryCode: CountryCode
  locale: LocaleCode
  currency: CurrencyCode
  timezone: RegionTimezone
  legalDocumentName: "CCCD" | "THAI_NATIONAL_ID"
  destinations: readonly string[]
  financials: {
    accountBalance: readonly [number, number]
    walletPayment: readonly [number, number]
    loanUnit: number
    booking: readonly [number, number]
    monthlyInterest: number
  }
  createPerson(random: Random, sequence: number, seed: number): SyntheticPerson
  airportForProvince(province: string): string
  walletDescription(sequence: number): string
  repaymentDescription(sequence: number): string
  ticketDescription(bookingReference: string): string
}
