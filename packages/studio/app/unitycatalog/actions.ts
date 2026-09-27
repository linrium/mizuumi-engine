"use server"

import { headers } from "next/headers"
import { auth } from "@/lib/auth"

type UnityCatalogObject = Record<string, unknown>

async function unityCatalogGet<T>(
  endpoint: string,
  accessToken: string,
  params?: Record<string, string>,
): Promise<T> {
  const baseUrl =
    process.env.UNITYCATALOG_URL ?? "https://unitycatalog.mizuumi.test"
  const url = new URL(
    `/api/2.1/unity-catalog${endpoint}`,
    baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`,
  )

  for (const [key, value] of Object.entries(params ?? {})) {
    url.searchParams.set(key, value)
  }

  const response = await fetch(url, {
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

export async function getUnityCatalogInfo(): Promise<{
  data?: UnityCatalogObject
  error?: string
}> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })

  if (!session) {
    return { error: "You must sign in with Keycloak to view Unity Catalog." }
  }

  try {
    // Better Auth stores the OAuth account in a signed cookie when no database
    // adapter is configured, and refreshes the token here when needed.
    const { accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: session.user.id },
    })

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

    return { data: { catalogs: catalogDetails } }
  } catch (error) {
    return {
      error:
        error instanceof Error
          ? error.message
          : "Unable to load Unity Catalog information.",
    }
  }
}
