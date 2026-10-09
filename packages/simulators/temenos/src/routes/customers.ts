import { eq } from "drizzle-orm";
import { Router } from "express";
import { db } from "../db/client.js";
import { accounts, customers, dataRights, products } from "../db/schema.js";
import { emit, EventTypes, Topics } from "../events/producer.js";

export const customersRouter = Router();

customersRouter.get("/customers/:customerId", (req, res) => {
  const row = db
    .select()
    .from(customers)
    .where(eq(customers.id, req.params.customerId))
    .get();
  if (!row) return res.status(404).json({ error: "customer_not_found" });

  emit(Topics.Customer, row.id, EventTypes.CustomerRead, {
    customerId: row.id,
    segment: row.segment,
    kycStatus: row.kycStatus,
    riskRating: row.riskRating,
  });
  res.json(row);
});

customersRouter.get("/customers/:customerId/data-rights", (req, res) => {
  const customer = db
    .select({ id: customers.id })
    .from(customers)
    .where(eq(customers.id, req.params.customerId))
    .get();
  if (!customer) return res.status(404).json({ error: "customer_not_found" });

  const rights = db
    .select()
    .from(dataRights)
    .where(eq(dataRights.customerId, req.params.customerId))
    .get();
  if (!rights) return res.status(404).json({ error: "data_rights_not_found" });

  emit(Topics.Customer, rights.customerId, EventTypes.CustomerDataRightsAccessed, {
    customerId: rights.customerId,
    consentVersion: rights.consentVersion,
    marketingOptIn: rights.marketingOptIn,
    thirdPartySharingOptIn: rights.thirdPartySharingOptIn,
  });
  res.json(rights);
});

customersRouter.get("/customers/:customerId/accounts", (req, res) => {
  const rows = db
    .select()
    .from(accounts)
    .where(eq(accounts.customerId, req.params.customerId))
    .all();

  emit(Topics.Customer, req.params.customerId, EventTypes.CustomerAccountsListed, {
    customerId: req.params.customerId,
    accountCount: rows.length,
    accountIds: rows.map((a) => a.id),
  });
  res.json({ customerId: req.params.customerId, accounts: rows });
});

customersRouter.get("/customers/:customerId/products", (req, res) => {
  const rows = db
    .select()
    .from(products)
    .where(eq(products.customerId, req.params.customerId))
    .all();

  emit(Topics.Customer, req.params.customerId, EventTypes.CustomerProductsListed, {
    customerId: req.params.customerId,
    productCount: rows.length,
    productCodes: rows.map((p) => p.productCode),
  });
  res.json({ customerId: req.params.customerId, products: rows });
});
