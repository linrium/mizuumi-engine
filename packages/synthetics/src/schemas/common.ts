import { z } from "zod"

const isoDatePattern = /^\d{4}-\d{2}-\d{2}$/
const isoDateTimePattern = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/

export const IsoDateSchema = z.string().regex(isoDatePattern, "Expected YYYY-MM-DD")
export const IsoDateTimeSchema = z
  .string()
  .regex(isoDateTimePattern, "Expected an ISO-8601 UTC timestamp")

export const VietnamesePhoneSchema = z
  .string()
  .regex(/^\+84[1-9]\d{8,9}$/, "Expected a Vietnamese phone number in +84 format")

export const ThaiPhoneSchema = z
  .string()
  .regex(/^\+66[689]\d{8}$/, "Expected a Thai phone number in +66 format")

export const RegionalPhoneSchema = z.union([VietnamesePhoneSchema, ThaiPhoneSchema])

export const VietnameseNationalIdSchema = z
  .string()
  .regex(/^\d{12}$/, "Vietnamese citizen identity numbers contain 12 digits")

export const ThaiNationalIdSchema = z
  .string()
  .regex(/^\d{13}$/, "Thai national identity numbers contain 13 digits")

export const RegionalNationalIdSchema = z.union([
  VietnameseNationalIdSchema,
  ThaiNationalIdSchema,
])

export const VietnameseAddressSchema = z.object({
  addressLine1: z.string().min(1),
  ward: z.string().min(1),
  district: z.string().min(1),
  province: z.string().min(1),
  postalCode: z.string().regex(/^\d{5,6}$/).optional(),
  countryCode: z.literal("VN"),
})

export const ThaiAddressSchema = z.object({
  addressLine1: z.string().min(1),
  ward: z.string().min(1),
  district: z.string().min(1),
  province: z.string().min(1),
  postalCode: z.string().regex(/^\d{5}$/).optional(),
  countryCode: z.literal("TH"),
})

export const RegionalAddressSchema = z.union([VietnameseAddressSchema, ThaiAddressSchema])

export const VietnamesePersonSchema = z.object({
  personId: z.string().min(1),
  fullName: z.string().min(1),
  fullNameAscii: z.string().min(1),
  familyName: z.string().min(1),
  middleName: z.string().min(1).optional(),
  givenName: z.string().min(1),
  dateOfBirth: IsoDateSchema,
  gender: z.enum(["FEMALE", "MALE", "OTHER", "UNSPECIFIED"]),
  nationalId: VietnameseNationalIdSchema,
  phoneNumber: VietnamesePhoneSchema,
  emailAddress: z.string().email(),
  nationalityCode: z.literal("VN"),
  residenceCountryCode: z.literal("VN"),
  preferredLanguage: z.literal("vi-VN"),
  address: VietnameseAddressSchema,
})

export const ThaiPersonSchema = z.object({
  personId: z.string().min(1),
  fullName: z.string().min(1),
  fullNameAscii: z.string().min(1),
  familyName: z.string().min(1),
  middleName: z.string().min(1).optional(),
  givenName: z.string().min(1),
  dateOfBirth: IsoDateSchema,
  gender: z.enum(["FEMALE", "MALE", "OTHER", "UNSPECIFIED"]),
  nationalId: ThaiNationalIdSchema,
  phoneNumber: ThaiPhoneSchema,
  emailAddress: z.string().email(),
  nationalityCode: z.literal("TH"),
  residenceCountryCode: z.literal("TH"),
  preferredLanguage: z.literal("th-TH"),
  address: ThaiAddressSchema,
})

export const SyntheticPersonSchema = z.discriminatedUnion("nationalityCode", [
  VietnamesePersonSchema,
  ThaiPersonSchema,
])

export const MoneySchema = z.object({
  amount: z.number().finite().nonnegative(),
  currency: z.enum(["VND", "THB"]),
})

export const OrganizationSchema = z.object({
  organizationId: z.enum(["vikki-bank", "hdbank", "vietjet-air", "group-crm"]),
  legalName: z.string().min(1),
  countryCode: z.enum(["VN", "TH"]),
  timezone: z.enum(["Asia/Ho_Chi_Minh", "Asia/Bangkok"]),
})

export type VietnamesePerson = z.infer<typeof VietnamesePersonSchema>
export type ThaiPerson = z.infer<typeof ThaiPersonSchema>
export type SyntheticPerson = z.infer<typeof SyntheticPersonSchema>
