import { z } from "zod"
import { OrganizationSchema } from "../common.js"
import {
  T24AccountSchema,
  T24CustomerSchema,
  T24FundsTransferSchema,
  T24LoanArrangementSchema,
} from "../temenos-t24.js"

export const HdbankDatasetSchema = z.object({
  organization: OrganizationSchema.extend({
    organizationId: z.literal("hdbank"),
    legalName: z.literal("Ngân hàng TMCP Phát triển Thành phố Hồ Chí Minh"),
  }),
  sourceSystem: z.literal("TEMENOS_T24"),
  customers: z.array(T24CustomerSchema),
  accounts: z.array(T24AccountSchema),
  loans: z.array(T24LoanArrangementSchema),
  fundsTransfers: z.array(T24FundsTransferSchema),
})

export type HdbankDataset = z.infer<typeof HdbankDatasetSchema>
