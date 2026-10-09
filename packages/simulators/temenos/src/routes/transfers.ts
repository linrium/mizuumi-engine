import { eq } from "drizzle-orm";
import { Router } from "express";
import { nanoid } from "nanoid";
import { z } from "zod";
import { db, sqlite } from "../db/client.js";
import { accounts, transfers } from "../db/schema.js";
import { emit, EventTypes, Topics } from "../events/producer.js";

export const transfersRouter = Router();

const TransferSchema = z.object({
  fromAccountId: z.string().optional(),
  toAccountId: z.string(),
  amount: z.number().positive(),
  currency: z.string().default("VND"),
  type: z.enum(["transfer", "cashback", "refund", "payout"]).default("transfer"),
  reference: z.string().optional(),
  description: z.string().optional(),
});

transfersRouter.post("/transfers", (req, res) => {
  const parse = TransferSchema.safeParse(req.body);
  if (!parse.success) {
    return res.status(400).json({ error: "invalid_payload", details: parse.error.flatten() });
  }
  const payload = parse.data;

  const id = `TRX-${nanoid(10)}`;

  emit(Topics.Payment, id, EventTypes.PaymentInitiated, {
    paymentId: id,
    fromAccountId: payload.fromAccountId ?? null,
    toAccountId: payload.toAccountId,
    amount: payload.amount,
    currency: payload.currency,
    type: payload.type,
    reference: payload.reference ?? null,
  });

  const toAccount = db.select().from(accounts).where(eq(accounts.id, payload.toAccountId)).get();
  if (!toAccount) {
    emit(Topics.Payment, id, EventTypes.PaymentFailed, {
      paymentId: id,
      reason: "to_account_not_found",
      toAccountId: payload.toAccountId,
    });
    return res.status(404).json({ error: "to_account_not_found" });
  }

  const requiresSource = payload.type === "transfer";
  const fromAccount = payload.fromAccountId
    ? db.select().from(accounts).where(eq(accounts.id, payload.fromAccountId)).get()
    : undefined;
  if (requiresSource && !fromAccount) {
    emit(Topics.Payment, id, EventTypes.PaymentFailed, {
      paymentId: id,
      reason: "from_account_not_found",
      fromAccountId: payload.fromAccountId ?? null,
    });
    return res.status(404).json({ error: "from_account_not_found" });
  }
  if (fromAccount && fromAccount.availableBalance < payload.amount) {
    emit(Topics.Payment, id, EventTypes.PaymentFailed, {
      paymentId: id,
      reason: "insufficient_funds",
      fromAccountId: fromAccount.id,
      requested: payload.amount,
      available: fromAccount.availableBalance,
    });
    return res.status(422).json({ error: "insufficient_funds" });
  }

  const now = new Date().toISOString();

  const txn = sqlite.transaction(() => {
    if (fromAccount) {
      const newBalance = fromAccount.balance - payload.amount;
      db.update(accounts)
        .set({ balance: newBalance, availableBalance: newBalance })
        .where(eq(accounts.id, fromAccount.id))
        .run();
    }
    const newToBalance = toAccount.balance + payload.amount;
    db.update(accounts)
      .set({ balance: newToBalance, availableBalance: newToBalance })
      .where(eq(accounts.id, toAccount.id))
      .run();

    db.insert(transfers)
      .values({
        id,
        fromAccountId: fromAccount?.id ?? null,
        toAccountId: toAccount.id,
        amount: payload.amount,
        currency: payload.currency,
        type: payload.type,
        reference: payload.reference,
        description: payload.description,
        status: "completed",
        completedAt: now,
      })
      .run();
  });
  txn();

  const created = db.select().from(transfers).where(eq(transfers.id, id)).get();
  emit(Topics.Payment, id, EventTypes.PaymentCompleted, created);
  res.status(201).json(created);
});

transfersRouter.get("/transfers/:transferId", (req, res) => {
  const row = db
    .select()
    .from(transfers)
    .where(eq(transfers.id, req.params.transferId))
    .get();
  if (!row) return res.status(404).json({ error: "transfer_not_found" });

  emit(Topics.Payment, row.id, EventTypes.PaymentRead, {
    paymentId: row.id,
    status: row.status,
    amount: row.amount,
    currency: row.currency,
    type: row.type,
  });
  res.json(row);
});
