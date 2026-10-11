"use server"

import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"

type UnityCatalogObject = Record<string, unknown>

function unityCatalogUrl(
  endpoint: string,
  params?: Record<string, string>,
): URL {
  const baseUrl =
    process.env.UNITYCATALOG_URL ?? "https://unitycatalog.mizuumi.test"
  const url = new URL(
    `/api/2.1/unity-catalog${endpoint}`,
    baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`,
  )
  for (const [key, value] of Object.entries(params ?? {})) {
    url.searchParams.set(key, value)
  }
  return url
}

async function unityCatalogGet<T>(
  endpoint: string,
  accessToken: string,
  params?: Record<string, string>,
): Promise<T> {
  const response = await fetch(unityCatalogUrl(endpoint, params), {
    headers: { Authorization: `Bearer ${accessToken}` },
    cache: "no-store",
  })
  const body: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    throw new Error(
      `Unity Catalog ${endpoint} returned ${response.status}: ${JSON.stringify(body)}`,
    )
  }

  return body as T
}

async function unityCatalogPatch<T>(
  endpoint: string,
  accessToken: string,
  payload: unknown,
): Promise<T> {
  const response = await fetch(unityCatalogUrl(endpoint), {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${accessToken}`,
      "Content-Type": "application/json",
    },
    cache: "no-store",
    body: JSON.stringify(payload),
  })
  const body: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    throw new Error(
      `Unity Catalog ${endpoint} returned ${response.status}: ${JSON.stringify(body)}`,
    )
  }

  return body as T
}

async function withAccessToken<T>(
  run: (accessToken: string) => Promise<T>,
): Promise<
  { ok: true; value: T } | { ok: false; needsReconnect?: true; error?: string }
> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")

  let accessToken: string
  try {
    ;({ accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: session.user.id },
    }))
  } catch {
    return { ok: false, needsReconnect: true }
  }

  try {
    return { ok: true, value: await run(accessToken) }
  } catch (error) {
    return {
      ok: false,
      error:
        error instanceof Error
          ? error.message
          : "Unity Catalog request failed.",
    }
  }
}

export type SecurableType = "catalog" | "schema" | "table"

export type PrivilegeAssignment = {
  principal: string
  privileges: string[]
}

export async function getUnityCatalogPermissions(
  securableType: SecurableType,
  fullName: string,
): Promise<{
  data?: { privilege_assignments?: PrivilegeAssignment[] }
  error?: string
  needsReconnect?: boolean
}> {
  const result = await withAccessToken((accessToken) =>
    unityCatalogGet<{ privilege_assignments?: PrivilegeAssignment[] }>(
      `/permissions/${securableType}/${encodeURIComponent(fullName)}`,
      accessToken,
    ),
  )
  if (!result.ok)
    return { error: result.error, needsReconnect: result.needsReconnect }
  return { data: result.value }
}

export async function updateUnityCatalogPermissions(input: {
  securableType: SecurableType
  fullName: string
  principal: string
  add?: string[]
  remove?: string[]
}): Promise<{
  data?: { privilege_assignments?: PrivilegeAssignment[] }
  error?: string
  needsReconnect?: boolean
}> {
  const { securableType, fullName, principal, add = [], remove = [] } = input
  if (!principal.trim()) return { error: "Principal is required." }
  if (add.length === 0 && remove.length === 0)
    return { error: "Select at least one privilege." }

  const result = await withAccessToken((accessToken) =>
    unityCatalogPatch<{ privilege_assignments?: PrivilegeAssignment[] }>(
      `/permissions/${securableType}/${encodeURIComponent(fullName)}`,
      accessToken,
      { changes: [{ principal, add, remove }] },
    ),
  )
  if (!result.ok)
    return { error: result.error, needsReconnect: result.needsReconnect }
  return { data: result.value }
}

export async function getUnityCatalogInfo(): Promise<{
  data?: UnityCatalogObject
  error?: string
  needsReconnect?: boolean
}> {
  const result = await withAccessToken(async (accessToken) => {
    const { catalogs = [] } = await unityCatalogGet<{
      catalogs?: UnityCatalogObject[]
    }>("/catalogs", accessToken, { max_results: "100" })

    const catalogDetails = await Promise.all(
      catalogs.map(async (catalog) => {
        const catalogName = String(catalog.name)
        const [details, { schemas = [] }] = await Promise.all([
          unityCatalogGet<UnityCatalogObject>(
            `/catalogs/${encodeURIComponent(catalogName)}`,
            accessToken,
          ),
          unityCatalogGet<{ schemas?: UnityCatalogObject[] }>(
            "/schemas",
            accessToken,
            { catalog_name: catalogName, max_results: "100" },
          ),
        ])

        const schemaDetails = await Promise.all(
          schemas.map(async (schema) => {
            const schemaName = String(schema.name)
            const [schemaInfo, { tables = [] }] = await Promise.all([
              unityCatalogGet<UnityCatalogObject>(
                `/schemas/${encodeURIComponent(`${catalogName}.${schemaName}`)}`,
                accessToken,
              ),
              unityCatalogGet<{ tables?: UnityCatalogObject[] }>(
                "/tables",
                accessToken,
                {
                  catalog_name: catalogName,
                  schema_name: schemaName,
                  max_results: "50",
                },
              ),
            ])

            return { ...schemaInfo, tables }
          }),
        )

        return { ...details, schemas: schemaDetails }
      }),
    )

    return { catalogs: catalogDetails }
  })

  if (!result.ok)
    return { error: result.error, needsReconnect: result.needsReconnect }
  return { data: result.value }
}
