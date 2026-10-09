import { eq } from "drizzle-orm";
import { Router } from "express";
import { db } from "../db/client.js";
import { accounts } from "../db/schema.js";
import { emit, EventTypes, Topics } from "../events/producer.js";

export const accountsRouter = Router();

accountsRouter.get("/accounts/:accountId", (req, res) => {
  const row = db
    .select()
    .from(accounts)
    .where(eq(accounts.id, req.params.accountId))
    .get();
  if (!row) return res.status(404).json({ error: "account_not_found" });

  emit(Topics.Account, row.id, EventTypes.AccountRead, {
    accountId: row.id,
    customerId: row.customerId,
    type: row.type,
    currency: row.currency,
    balance: row.balance,
    availableBalance: row.availableBalance,
    status: row.status,
  });
  res.json(row);
});
