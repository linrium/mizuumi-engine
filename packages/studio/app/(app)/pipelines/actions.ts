"use server"

import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"

export type ExecutorStateSummary = {
  running: number
  completed: number
  failed: number
  pending: number
  unknown: number
}

export type Pipeline = {
  name: string
  namespace: string
  state: string
  application_state_message?: string | null
  spark_version?: string | null
  application_type?: string | null
  mode?: string | null
  image?: string | null
  main_application_file?: string | null
  submission_attempts?: number | null
  execution_attempts?: number | null
  submission_time?: string | null
  termination_time?: string | null
  last_submission_attempt_time?: string | null
  creation_timestamp?: string | null
  driver_pod_name?: string | null
  executor_instances?: number | null
  executor_state_summary: ExecutorStateSummary
}

export type ListPipelinesResponse = {
  items: Pipeline[]
}

export type PipelineDetail = Pipeline & { raw: unknown }

function controlPlaneUrl(path: string, params?: Record<string, string>): URL {
  const baseUrl =
    process.env.CONTROLPLANE_URL ?? "https://api.controlplane.mizuumi.test"
  const url = new URL(path, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`)
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value) url.searchParams.set(key, value)
  }
  return url
}

async function request<T>(
  path: string,
  params?: Record<string, string>,
): Promise<T> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")
  const { accessToken } = await auth.api.getAccessToken({
    headers: requestHeaders,
    body: { useAccountCookie: true, userId: session.user.id },
  })
  const response = await fetch(controlPlaneUrl(path, params), {
    cache: "no-store",
    headers: {
      Accept: "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
  })
  const body: unknown = await response.json().catch(() => null)
  if (!response.ok) {
    const message =
      body && typeof body === "object" && "error" in body
        ? String((body as { error: unknown }).error)
        : `HTTP ${response.status}`
    throw new Error(`Control plane ${path}: ${message}`)
  }
  return body as T
}

export async function listPipelines(
  namespace?: string,
): Promise<ListPipelinesResponse> {
  return request<ListPipelinesResponse>(
    "api/pipelines",
    namespace ? { namespace } : undefined,
  )
}

export async function getPipeline(
  namespace: string,
  name: string,
): Promise<PipelineDetail> {
  return request<PipelineDetail>(
    `api/pipelines/${encodeURIComponent(namespace)}/${encodeURIComponent(name)}`,
  )
}
