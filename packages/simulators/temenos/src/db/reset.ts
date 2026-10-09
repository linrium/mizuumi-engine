import { existsSync, unlinkSync } from "node:fs";

const DB_PATH = process.env.TEMENOS_DB_PATH ?? "./temenos.db";

for (const suffix of ["", "-wal", "-shm", "-journal"]) {
  const file = `${DB_PATH}${suffix}`;
  if (existsSync(file)) {
    unlinkSync(file);
    console.log(`Removed ${file}`);
  }
}

const { runMigrations } = await import("./migrate.js");
const { seedDatabase } = await import("./seed.js");

runMigrations();
seedDatabase();

console.log("Temenos simulator database reset.");
