"use server"

import { revalidatePath } from "next/cache"
import { headers } from "next/headers"
import { redirect } from "next/navigation"
import { auth } from "@/lib/auth"
import type {
  AccessRequest,
  ActionResult,
  CreatePolicyTemplateInput,
  PolicyTemplate,
  PolicyTemplateSummary,
} from "./types"

type SessionUser = {
  id: string
  name?: string | null
  email?: string | null
}

type AccessRequestSummary = Pick<AccessRequest, "id" | "submitted_at">

type OrganizationClaim = Record<string, { groups?: unknown } | null | undefined>

function controlPlaneUrl(
  endpoint: string,
  params?: Record<string, string>,
): URL {
  const baseUrl =
    process.env.CONTROLPLANE_URL ?? "https://api.controlplane.mizuumi.test"
  const url = new URL(endpoint, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`)
  for (const [key, value] of Object.entries(params ?? {})) {
    url.searchParams.set(key, value)
  }
  return url
}

async function controlPlaneRequest<T>(
  endpoint: string,
  init?: RequestInit,
  params?: Record<string, string>,
): Promise<T> {
  const response = await fetch(controlPlaneUrl(endpoint, params), {
    ...init,
    cache: "no-store",
    headers: {
      Accept: "application/json",
      ...(init?.body ? { "Content-Type": "application/json" } : {}),
      ...init?.headers,
    },
  })
  const body: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    const message =
      body && typeof body === "object" && "error" in body
        ? String(body.error)
        : `HTTP ${response.status}`
    throw new Error(`Control plane ${endpoint}: ${message}`)
  }

  return body as T
}

async function currentUser(): Promise<{
  requestHeaders: Awaited<ReturnType<typeof headers>>
  user: SessionUser
}> {
  const requestHeaders = await headers()
  const session = await auth.api.getSession({ headers: requestHeaders })
  if (!session) redirect("/login")
  return { requestHeaders, user: session.user }
}

function principal(user: SessionUser): string {
  return user.email ?? user.name ?? user.id
}

function groupsFromOrganizationClaim(claim: unknown): string[] {
  if (!claim || typeof claim !== "object" || Array.isArray(claim)) return []

  return Object.values(claim as OrganizationClaim)
    .flatMap((membership) =>
      Array.isArray(membership?.groups)
        ? membership.groups
            .filter(
              (group): group is string =>
                typeof group === "string" && group.trim().length > 0,
            )
            .map((group) => group.trim().replace(/^\/+|\/+$/g, ""))
        : [],
    )
    .filter(Boolean)
}

async function currentApprovalTeams(
  requestHeaders: Awaited<ReturnType<typeof headers>>,
  userId: string,
): Promise<string[]> {
  const { accessToken } = await auth.api.getAccessToken({
    headers: requestHeaders,
    body: { useAccountCookie: true, userId },
  })
  const issuer = (
    process.env.KEYCLOAK_ISSUER ?? "https://auth.mizuumi.test/realms/sovico"
  ).replace(/\/$/, "")
  const response = await fetch(`${issuer}/protocol/openid-connect/userinfo`, {
    headers: { Authorization: `Bearer ${accessToken}` },
    cache: "no-store",
  })
  if (!response.ok) {
    throw new Error(`Keycloak userinfo returned HTTP ${response.status}`)
  }

  const userInfo = (await response.json()) as { organization?: unknown }
  return [...new Set(groupsFromOrganizationClaim(userInfo.organization))].sort()
}

function errorResult<T>(error: unknown): ActionResult<T> {
  return {
    ok: false,
    error:
      error instanceof Error ? error.message : "Control plane request failed.",
  }
}

export async function listPolicyTemplates(): Promise<
  ActionResult<PolicyTemplateSummary[]>
> {
  await currentUser()
  try {
    const result = await controlPlaneRequest<{
      policy_templates: PolicyTemplateSummary[]
    }>("/api/policy-templates")
    return { ok: true, data: result.policy_templates }
  } catch (error) {
    return errorResult(error)
  }
}

export async function getPolicyTemplate(
  templateId: string,
): Promise<ActionResult<PolicyTemplate>> {
  await currentUser()
  try {
    const template = await controlPlaneRequest<PolicyTemplate>(
      `/api/policy-templates/${encodeURIComponent(templateId)}`,
    )
    return { ok: true, data: template }
  } catch (error) {
    return errorResult(error)
  }
}

export async function createPolicyTemplate(
  input: CreatePolicyTemplateInput,
): Promise<ActionResult<PolicyTemplate>> {
  const { user } = await currentUser()
  try {
    const template = await controlPlaneRequest<PolicyTemplate>(
      "/api/policy-templates",
      {
        method: "POST",
        body: JSON.stringify({
          ...input,
          description: input.description || null,
          default_duration_seconds: input.default_duration_seconds ?? null,
          max_duration_seconds: input.max_duration_seconds ?? null,
          guardrails: [],
          created_by: principal(user),
        }),
      },
    )
    revalidatePath("/review/policy-templates")
    return { ok: true, data: template }
  } catch (error) {
    return errorResult(error)
  }
}

export async function listRequestQueue(): Promise<
  ActionResult<{ requests: AccessRequest[]; approvalTeams: string[] }>
> {
  const { requestHeaders, user } = await currentUser()
  try {
    const approvalTeams = await currentApprovalTeams(requestHeaders, user.id)
    const responses = await Promise.all(
      approvalTeams.map((team) =>
        controlPlaneRequest<{ access_requests: AccessRequestSummary[] }>(
          "/api/access-requests",
          undefined,
          { approval_team: team },
        ),
      ),
    )
    const queuedRequests = [
      ...new Map(
        responses
          .flatMap((response) => response.access_requests)
          .map((request) => [request.id, request]),
      ).values(),
    ]
    const requests = await Promise.all(
      queuedRequests.map((request) =>
        controlPlaneRequest<AccessRequest>(
          `/api/access-requests/${encodeURIComponent(request.id)}`,
        ),
      ),
    )
    requests.sort((left, right) =>
      right.submitted_at.localeCompare(left.submitted_at),
    )

    return { ok: true, data: { requests, approvalTeams } }
  } catch (error) {
    return errorResult(error)
  }
}

export async function reviewAccessRequest(input: {
  requestId: string
  approvalTeam: string
  decision: "approve" | "reject"
  comment?: string
}): Promise<ActionResult<AccessRequest>> {
  const { requestHeaders, user } = await currentUser()
  try {
    const approvalTeams = await currentApprovalTeams(requestHeaders, user.id)
    if (!approvalTeams.includes(input.approvalTeam)) {
      return { ok: false, error: "You are not a member of that approval team." }
    }

    const request = await controlPlaneRequest<AccessRequest>(
      `/api/access-requests/${encodeURIComponent(input.requestId)}/${input.decision}`,
      {
        method: "POST",
        body: JSON.stringify({
          approval_team: input.approvalTeam,
          reviewer: principal(user),
          comment: input.comment || null,
        }),
      },
    )
    revalidatePath("/review/requests")
    return { ok: true, data: request }
  } catch (error) {
    return errorResult(error)
  }
}
