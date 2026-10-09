import express from "express";
import { runMigrations } from "./db/migrate.js";
import { shutdownProducer } from "./events/producer.js";
import { accountsRouter } from "./routes/accounts.js";
import { customersRouter } from "./routes/customers.js";
import { eligibilityRouter } from "./routes/eligibility.js";
import { transfersRouter } from "./routes/transfers.js";

runMigrations();

const app = express();
app.use(express.json());

app.get("/health", (_req, res) => {
  res.json({ status: "ok", service: "temenos-simulator", time: new Date().toISOString() });
});

app.use("/mock", customersRouter);
app.use("/mock", accountsRouter);
app.use("/mock", eligibilityRouter);
app.use("/mock", transfersRouter);

app.use((err: Error, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
  console.error(err);
  res.status(500).json({ error: "internal_error", message: err.message });
});

const PORT = Number(process.env.PORT ?? 4010);
const server = app.listen(PORT, () => {
  console.log(`Temenos T24 simulator listening on http://localhost:${PORT}`);
});

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, async () => {
    console.log(`\n[${signal}] shutting down…`);
    await shutdownProducer();
    server.close(() => process.exit(0));
  });
}
