"use server"

import { revalidatePath } from "next/cache"
import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import type {
  ContractActionResult,
  ContractTarget,
  DataContract,
  DataQualityRun,
  QualityRunActionResult,
  QualityRunsActionResult,
} from "./types"

function controlPlaneUrl(endpoint: string): URL {
  const baseUrl =
    process.env.CONTROLPLANE_URL ?? "https://api.controlplane.mizuumi.test"
  return new URL(endpoint, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`)
}

function fullName(target: ContractTarget): string {
  const parts = [target.catalog, target.schema, target.table]
  if (parts.some((part) => !part.trim() || part.includes("."))) {
    throw new Error("Invalid Unity Catalog table name.")
  }
  return parts.join(".")
}

async function requireUser() {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")
  return { requestHeaders, user: session.user }
}

async function requestContract(
  target: ContractTarget,
  method: "GET" | "POST",
  accessToken: string,
): Promise<DataContract | null> {
  const name = fullName(target)
  const suffix = method === "POST" ? "/reconcile" : ""
  const endpoint = `/api/data-contracts/${encodeURIComponent(name)}${suffix}`
  const response = await fetch(controlPlaneUrl(endpoint), {
    method,
    cache: "no-store",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
  })
  const body: unknown = await response.json().catch(() => null)

  if (method === "GET" && response.status === 404) return null
  if (!response.ok) {
    const message =
      body && typeof body === "object" && "error" in body
        ? String(body.error)
        : `HTTP ${response.status}`
    throw new Error(`Control plane ${endpoint}: ${message}`)
  }

  return body as DataContract
}

function errorResult(error: unknown): ContractActionResult {
  return {
    ok: false,
    error:
      error instanceof Error ? error.message : "Data contract request failed.",
  }
}

export async function getDataContract(
  target: ContractTarget,
): Promise<ContractActionResult> {
  const { requestHeaders, user } = await requireUser()
  try {
    const { accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: user.id },
    })
    return {
      ok: true,
      data: await requestContract(target, "GET", accessToken),
    }
  } catch (error) {
    return errorResult(error)
  }
}

export async function getDataQualityRuns(
  target: ContractTarget,
): Promise<QualityRunsActionResult> {
  const { requestHeaders, user } = await requireUser()
  try {
    const { accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: user.id },
    })
    const endpoint = `/api/data-contracts/${encodeURIComponent(fullName(target))}/quality-runs`
    const response = await fetch(controlPlaneUrl(endpoint), {
      cache: "no-store",
      headers: {
        Accept: "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
    })
    const body: unknown = await response.json().catch(() => null)
    if (!response.ok) throw controlPlaneError(endpoint, response.status, body)
    return {
      ok: true,
      data: (body as { quality_runs?: DataQualityRun[] }).quality_runs ?? [],
    }
  } catch (error) {
    return { ok: false, error: errorMessage(error) }
  }
}

export async function runDataQuality(
  target: ContractTarget,
): Promise<QualityRunActionResult> {
  const { requestHeaders, user } = await requireUser()
  try {
    const { accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: user.id },
    })
    const endpoint = `/api/data-contracts/${encodeURIComponent(fullName(target))}/quality-runs`
    const response = await fetch(controlPlaneUrl(endpoint), {
      method: "POST",
      cache: "no-store",
      headers: {
        Accept: "application/json",
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        triggered_by: user.email ?? user.name ?? user.id,
      }),
      signal: AbortSignal.timeout(90_000),
    })
    const body: unknown = await response.json().catch(() => null)
    if (!response.ok) throw controlPlaneError(endpoint, response.status, body)
    revalidatePath(
      `/unitycatalog/${encodeURIComponent(target.catalog)}/${encodeURIComponent(target.schema)}/${encodeURIComponent(target.table)}/contract`,
    )
    return { ok: true, data: body as DataQualityRun }
  } catch (error) {
    return { ok: false, error: errorMessage(error) }
  }
}

export async function reconcileDataContract(
  target: ContractTarget,
): Promise<ContractActionResult> {
  const { requestHeaders, user } = await requireUser()
  try {
    const { accessToken } = await auth.api.getAccessToken({
      headers: requestHeaders,
      body: { useAccountCookie: true, userId: user.id },
    })
    const data = await requestContract(target, "POST", accessToken)
    revalidatePath(
      `/unitycatalog/${encodeURIComponent(target.catalog)}/${encodeURIComponent(target.schema)}/${encodeURIComponent(target.table)}/contract`,
    )
    return { ok: true, data }
  } catch (error) {
    return errorResult(error)
  }
}

function controlPlaneError(endpoint: string, status: number, body: unknown) {
  const message =
    body && typeof body === "object" && "error" in body
      ? String(body.error)
      : `HTTP ${status}`
  return new Error(`Control plane ${endpoint}: ${message}`)
}

function errorMessage(error: unknown) {
  return error instanceof Error
    ? error.message
    : "Control plane request failed."
}
