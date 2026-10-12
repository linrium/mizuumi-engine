import { z } from "zod"
import { IsoDateSchema, OrganizationSchema } from "../common.js"
import {
  T24AccountSchema,
  T24CustomerSchema,
  T24FundsTransferSchema,
} from "../temenos-t24.js"

export const VikkiWalletSchema = z.object({
  walletId: z.string().min(1),
  customerId: z.string().min(1),
  settlementAccountNumber: z.string().min(1),
  walletType: z.literal("VIKKI_DIGITAL_WALLET"),
  activatedAt: IsoDateSchema,
  status: z.enum(["ACTIVE", "SUSPENDED", "CLOSED"]),
})

export const VikkiBankDatasetSchema = z.object({
  organization: OrganizationSchema.extend({
    organizationId: z.literal("vikki-bank"),
    legalName: z.literal("Ngân hàng TNHH MTV Số Vikki"),
  }),
  sourceSystem: z.literal("TEMENOS_T24"),
  customers: z.array(T24CustomerSchema),
  accounts: z.array(T24AccountSchema),
  wallets: z.array(VikkiWalletSchema),
  fundsTransfers: z.array(T24FundsTransferSchema),
})

export type VikkiBankDataset = z.infer<typeof VikkiBankDatasetSchema>
export type VikkiWallet = z.infer<typeof VikkiWalletSchema>
