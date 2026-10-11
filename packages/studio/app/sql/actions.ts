"use server"

import { type DuckDBConnection, DuckDBInstance } from "@duckdb/node-api"
import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"

export type SqlQueryResult = {
  columns?: string[]
  columnTypes?: string[]
  rows?: unknown[][]
  elapsedMs?: number
  error?: string
}

const MAX_SQL_LENGTH = 100_000

let duckDBInstance: Promise<DuckDBInstance> | undefined

async function getDuckDBInstance() {
  duckDBInstance ??= DuckDBInstance.create(":memory:")
  return duckDBInstance
}

export async function executeSql(sql: string): Promise<SqlQueryResult> {
  const session = await auth.api.getSession({ headers: await headers() })

  if (!session) {
    redirect("/login")
  }

  const query = sql.trim()
  if (!query) return { error: "Enter a SQL query to execute." }
  if (query.length > MAX_SQL_LENGTH) {
    return {
      error: `SQL is limited to ${MAX_SQL_LENGTH.toLocaleString()} characters.`,
    }
  }

  const token = process.env.QUACK_TOKEN
  if (!token) {
    return { error: "Studio is missing the QUACK_TOKEN environment variable." }
  }

  const uri = process.env.QUACK_URI ?? "quack:query.mizuumi.test:443"
  const caFile = process.env.QUACK_CA_FILE ?? "../../k8s/auth/tls/ca.crt"
  const startedAt = performance.now()
  let connection: DuckDBConnection | undefined

  try {
    connection = await (await getDuckDBInstance()).connect()
    await connection.run("INSTALL quack; LOAD quack")
    await connection.run("SET ca_cert_file = ?", [caFile])
    await connection.run("SET enable_server_cert_verification = true")

    const reader = await connection.runAndReadAll(
      "FROM quack_query(?, ?, token = ?)",
      [uri, query, token],
    )

    return {
      columns: reader.columnNames(),
      columnTypes: reader.columnTypes().map((type) => type.toString()),
      rows: reader.getRowsJson(),
      elapsedMs: Math.round(performance.now() - startedAt),
    }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : "Unable to execute SQL.",
      elapsedMs: Math.round(performance.now() - startedAt),
    }
  } finally {
    connection?.closeSync()
  }
}
