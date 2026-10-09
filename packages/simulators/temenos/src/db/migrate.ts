import { migrate } from "drizzle-orm/better-sqlite3/migrator";
import { db } from "./client.js";

export function runMigrations() {
  migrate(db, { migrationsFolder: "./drizzle" });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runMigrations();
  console.log("Applied Drizzle migrations.");
}
