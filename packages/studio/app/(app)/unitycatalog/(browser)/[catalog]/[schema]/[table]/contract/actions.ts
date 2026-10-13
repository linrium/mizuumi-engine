"use server"

import { revalidatePath } from "next/cache"
import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import type {
  ContractActionResult,
  ContractTarget,
  DataContract,
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

async function requireUser(): Promise<void> {
  const session = await auth.api.getSession({ headers: await headers() })
  if (!session) redirect("/login")
}

async function requestContract(
  target: ContractTarget,
  method: "GET" | "POST",
): Promise<DataContract | null> {
  const name = fullName(target)
  const suffix = method === "POST" ? "/reconcile" : ""
  const endpoint = `/api/data-contracts/${encodeURIComponent(name)}${suffix}`
  const response = await fetch(controlPlaneUrl(endpoint), {
    method,
    cache: "no-store",
    headers: { Accept: "application/json" },
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
  await requireUser()
  try {
    return { ok: true, data: await requestContract(target, "GET") }
  } catch (error) {
    return errorResult(error)
  }
}

export async function reconcileDataContract(
  target: ContractTarget,
): Promise<ContractActionResult> {
  await requireUser()
  try {
    const data = await requestContract(target, "POST")
    revalidatePath(
      `/unitycatalog/${encodeURIComponent(target.catalog)}/${encodeURIComponent(target.schema)}/${encodeURIComponent(target.table)}/contract`,
    )
    return { ok: true, data }
  } catch (error) {
    return errorResult(error)
  }
}
