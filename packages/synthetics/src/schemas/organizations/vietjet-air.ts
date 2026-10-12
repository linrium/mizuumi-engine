import { z } from "zod"
import {
  IsoDateSchema,
  IsoDateTimeSchema,
  MoneySchema,
  OrganizationSchema,
  RegionalNationalIdSchema,
  RegionalPhoneSchema,
} from "../common.js"

export const VietjetCustomerSchema = z.object({
  customerId: z.string().min(1),
  personId: z.string().min(1),
  fullName: z.string().min(1),
  dateOfBirth: IsoDateSchema,
  gender: z.enum(["FEMALE", "MALE", "OTHER", "UNSPECIFIED"]),
  nationalId: RegionalNationalIdSchema,
  nationalityCode: z.enum(["VN", "TH"]),
  phoneNumber: RegionalPhoneSchema,
  emailAddress: z.string().email(),
  preferredLanguage: z.enum(["vi-VN", "th-TH"]),
  skyJoyMemberNumber: z.string().min(1).optional(),
})

export const VietjetFlightSegmentSchema = z.object({
  flightNumber: z.string().regex(/^VJ\d{2,4}$/),
  departureAirport: z.string().regex(/^[A-Z]{3}$/),
  arrivalAirport: z.string().regex(/^[A-Z]{3}$/),
  scheduledDepartureAt: IsoDateTimeSchema,
  scheduledArrivalAt: IsoDateTimeSchema,
  fareClass: z.string().min(1),
  seatNumber: z.string().regex(/^\d{1,2}[A-F]$/).optional(),
})

export const VietjetBookingSchema = z.object({
  bookingId: z.string().min(1),
  bookingReference: z.string().regex(/^[A-Z0-9]{6}$/),
  customerId: z.string().min(1),
  bookedAt: IsoDateTimeSchema,
  status: z.enum(["CONFIRMED", "FLOWN", "CANCELLED", "REFUNDED"]),
  passengers: z
    .array(
      z.object({
        passengerId: z.string().min(1),
        fullName: z.string().min(1),
        passengerType: z.enum(["ADULT", "CHILD", "INFANT"]),
      }),
    )
    .min(1),
  segments: z.array(VietjetFlightSegmentSchema).min(1),
  totalAmount: MoneySchema,
  payment: z.object({
    paymentId: z.string().min(1),
    method: z.enum(["VIKKI_WALLET", "BANK_CARD", "BANK_TRANSFER", "CASH"]),
    status: z.enum(["PENDING", "PAID", "FAILED", "REFUNDED"]),
    paidAt: IsoDateTimeSchema.optional(),
    externalTransactionReference: z.string().min(1).optional(),
  }),
})

export const VietjetAirDatasetSchema = z.object({
  organization: OrganizationSchema.extend({
    organizationId: z.literal("vietjet-air"),
    legalName: z.literal("Công ty Cổ phần Hàng không VietJet"),
  }),
  sourceSystem: z.literal("VIETJET_RESERVATION"),
  customers: z.array(VietjetCustomerSchema),
  bookings: z.array(VietjetBookingSchema),
})

export type VietjetAirDataset = z.infer<typeof VietjetAirDatasetSchema>
export type VietjetCustomer = z.infer<typeof VietjetCustomerSchema>
export type VietjetBooking = z.infer<typeof VietjetBookingSchema>
