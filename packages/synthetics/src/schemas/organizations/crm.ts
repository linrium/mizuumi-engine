import { z } from "zod"
import {
  IsoDateSchema,
  IsoDateTimeSchema,
  OrganizationSchema,
  RegionalAddressSchema,
  RegionalNationalIdSchema,
  RegionalPhoneSchema,
} from "../common.js"

export const CrmContactSchema = z.object({
  contactId: z.string().min(1),
  personId: z.string().min(1),
  sourceRecordId: z.string().min(1),
  fullName: z.string().min(1),
  fullNameAscii: z.string().min(1),
  dateOfBirth: IsoDateSchema.optional(),
  nationalId: RegionalNationalIdSchema.optional(),
  phoneNumber: RegionalPhoneSchema,
  emailAddress: z.string().email(),
  preferredLanguage: z.enum(["vi-VN", "th-TH"]),
  countryCode: z.enum(["VN", "TH"]),
  address: RegionalAddressSchema.optional(),
  lifecycleStage: z.enum(["LEAD", "PROSPECT", "CUSTOMER", "INACTIVE"]),
  status: z.enum(["ACTIVE", "DO_NOT_CONTACT", "ARCHIVED"]),
  createdAt: IsoDateTimeSchema,
  updatedAt: IsoDateTimeSchema,
})

export const CrmCampaignInteractionSchema = z.object({
  interactionId: z.string().min(1),
  contactId: z.string().min(1),
  campaignId: z.string().min(1),
  campaignName: z.string().min(1),
  channel: z.enum(["EMAIL", "SMS", "PUSH_NOTIFICATION", "PHONE_CALL"]),
  direction: z.enum(["INBOUND", "OUTBOUND"]),
  outcome: z.enum([
    "DELIVERED",
    "OPENED",
    "CLICKED",
    "RESPONDED",
    "BOUNCED",
    "UNSUBSCRIBED",
    "NO_ANSWER",
  ]),
  occurredAt: IsoDateTimeSchema,
})

export const CrmPollBatchSchema = z
  .object({
    pollId: z.string().min(1),
    startedAt: IsoDateTimeSchema,
    completedAt: IsoDateTimeSchema.optional(),
    cursor: z.string().min(1).optional(),
    status: z.enum(["RUNNING", "COMPLETED", "FAILED"]),
    recordsRead: z.number().int().nonnegative(),
  })
  .superRefine((poll, context) => {
    if (poll.status === "COMPLETED" && !poll.completedAt) {
      context.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["completedAt"],
        message: "A completed CRM poll must include completedAt",
      })
    }
  })

export const CrmDatasetSchema = z.object({
  organization: OrganizationSchema.extend({
    organizationId: z.literal("group-crm"),
    legalName: z.literal("Sovico Group CRM"),
  }),
  sourceSystem: z.literal("GROUP_CRM"),
  pollBatches: z.array(CrmPollBatchSchema),
  contacts: z.array(CrmContactSchema),
  campaignInteractions: z.array(CrmCampaignInteractionSchema),
})

export type CrmContact = z.infer<typeof CrmContactSchema>
export type CrmCampaignInteraction = z.infer<typeof CrmCampaignInteractionSchema>
export type CrmPollBatch = z.infer<typeof CrmPollBatchSchema>
export type CrmDataset = z.infer<typeof CrmDatasetSchema>
