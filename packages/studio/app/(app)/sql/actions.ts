"use server"

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

export async function executeSql(sql: string): Promise<SqlQueryResult> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })

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

  let accessToken: string
  try {
    ;({ accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: session.user.id },
    }))
  } catch {
    return { error: "Reconnect your Keycloak account before running SQL." }
  }

  const queryUrl =
    process.env.DUCK_QUERY_URL ?? "https://query.mizuumi.test/api/query"
  const startedAt = performance.now()

  try {
    const response = await fetch(queryUrl, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ sql: query }),
      cache: "no-store",
      signal: AbortSignal.timeout(60_000),
    })
    const body = (await response.json()) as SqlQueryResult
    if (!response.ok) {
      return {
        error: body.error ?? `Duck query service returned ${response.status}.`,
        elapsedMs: Math.round(performance.now() - startedAt),
      }
    }
    return { ...body, elapsedMs: Math.round(performance.now() - startedAt) }
  } catch (error) {
    return {
      error: error instanceof Error ? error.message : "Unable to execute SQL.",
      elapsedMs: Math.round(performance.now() - startedAt),
    }
  }
}
