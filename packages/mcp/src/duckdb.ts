import { DuckDBInstance, type DuckDBConnection } from "@duckdb/node-api";
import { config } from "./config.js";

let instance: Promise<DuckDBInstance> | undefined;

async function getInstance(): Promise<DuckDBInstance> {
  instance ??= DuckDBInstance.create(":memory:");
  return instance;
}

export interface QueryResult {
  columns: string[];
  columnTypes: string[];
  rows: unknown[][];
  rowCount: number;
}

export async function runSql(sql: string): Promise<QueryResult> {
  let connection: DuckDBConnection | undefined;
  try {
    connection = await (await getInstance()).connect();
    // INSTALL/LOAD must happen before we pin ca_cert_file to the gateway CA,
    // otherwise the TLS handshake with extensions.duckdb.org fails.
    await connection.run("INSTALL quack");
    await connection.run("LOAD quack");
    if (config.gatewayCaFile) {
      await connection.run("SET ca_cert_file = ?", [config.gatewayCaFile]);
      await connection.run("SET enable_server_cert_verification = true");
    }
    const reader = await connection.runAndReadAll(
      "FROM quack_query(?, ?, token = ?)",
      [config.quackUri, sql, config.quackToken],
    );
    const rows = reader.getRowsJson() as unknown[][];
    return {
      columns: reader.columnNames(),
      columnTypes: reader.columnTypes().map((type) => type.toString()),
      rows,
      rowCount: rows.length,
    };
  } finally {
    connection?.closeSync();
  }
}

export async function warmup(): Promise<void> {
  await runSql("SELECT 1");
}
