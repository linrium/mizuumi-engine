import { eq } from "drizzle-orm";
import { Router } from "express";
import { z } from "zod";
import { db } from "../db/client.js";
import { accounts, customers, products } from "../db/schema.js";
import { emit, EventTypes, Topics } from "../events/producer.js";

export const eligibilityRouter = Router();

const EvaluateSchema = z.object({
  customerId: z.string(),
  campaignId: z.string(),
  rules: z
    .object({
      minBalance: z.number().optional(),
      requiredSegment: z.string().optional(),
      requiredKycStatus: z.string().optional(),
      maxRiskRating: z.enum(["low", "medium", "high"]).optional(),
      requiredProductCategory: z.string().optional(),
      excludedProductCategory: z.string().optional(),
    })
    .optional(),
});

const RISK_ORDER: Record<string, number> = { low: 1, medium: 2, high: 3 };

eligibilityRouter.post("/eligibility/evaluate", (req, res) => {
  const parse = EvaluateSchema.safeParse(req.body);
  if (!parse.success) {
    return res.status(400).json({ error: "invalid_payload", details: parse.error.flatten() });
  }
  const { customerId, campaignId, rules = {} } = parse.data;

  const customer = db.select().from(customers).where(eq(customers.id, customerId)).get();
  if (!customer) {
    return res.status(404).json({ error: "customer_not_found" });
  }

  const customerAccounts = db.select().from(accounts).where(eq(accounts.customerId, customerId)).all();
  const customerProducts = db.select().from(products).where(eq(products.customerId, customerId)).all();
  const totalBalance = customerAccounts.reduce((sum, a) => sum + a.balance, 0);

  const reasons: string[] = [];

  if (rules.minBalance !== undefined && totalBalance < rules.minBalance) {
    reasons.push(`total_balance_below_min:${totalBalance}<${rules.minBalance}`);
  }
  if (rules.requiredSegment && customer.segment !== rules.requiredSegment) {
    reasons.push(`segment_mismatch:${customer.segment}!=${rules.requiredSegment}`);
  }
  if (rules.requiredKycStatus && customer.kycStatus !== rules.requiredKycStatus) {
    reasons.push(`kyc_mismatch:${customer.kycStatus}!=${rules.requiredKycStatus}`);
  }
  if (rules.maxRiskRating) {
    const current = RISK_ORDER[customer.riskRating ?? "low"] ?? 1;
    const max = RISK_ORDER[rules.maxRiskRating];
    if (current > max) reasons.push(`risk_too_high:${customer.riskRating}>${rules.maxRiskRating}`);
  }
  if (rules.requiredProductCategory) {
    const has = customerProducts.some((p) => p.category === rules.requiredProductCategory);
    if (!has) reasons.push(`missing_product_category:${rules.requiredProductCategory}`);
  }
  if (rules.excludedProductCategory) {
    const has = customerProducts.some((p) => p.category === rules.excludedProductCategory);
    if (has) reasons.push(`has_excluded_product_category:${rules.excludedProductCategory}`);
  }

  const eligible = reasons.length === 0;

  const payload = {
    campaignId,
    customerId,
    eligible,
    reasons,
    evaluatedAt: new Date().toISOString(),
    context: {
      segment: customer.segment,
      kycStatus: customer.kycStatus,
      riskRating: customer.riskRating,
      totalBalance,
      accountCount: customerAccounts.length,
      productCount: customerProducts.length,
    },
  };

  emit(Topics.Eligibility, `${customerId}:${campaignId}`, EventTypes.EligibilityEvaluated, payload, {
    subject: campaignId,
  });
  res.json(payload);
});
