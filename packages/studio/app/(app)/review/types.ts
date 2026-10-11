export type ApprovalTeam = {
  team: string
  required_approvals: number
}

export type PolicyGrant = {
  securable_type: string
  target_kind: "REQUEST_RESOURCE" | "FIXED_RESOURCE"
  target_full_name: string
  privilege: string
}

export type Guardrail = {
  guardrail_key: string
  name: string
  description?: string | null
  enforcement: "REQUIRED" | "ADVISORY"
  configuration: Record<string, unknown>
}

export type PolicyTemplateSummary = {
  id: string
  key: string
  version: number
  name: string
  description?: string | null
  approval_strategy: "ANY" | "ALL"
  default_duration_seconds?: number | null
  max_duration_seconds?: number | null
  created_by: string
  created_at: string
}

export type PolicyTemplate = PolicyTemplateSummary & {
  status: "DRAFT" | "PUBLISHED" | "RETIRED"
  blast_radius_required: boolean
  guardrail_review_required: boolean
  affected_teams: string[]
  approval_teams: ApprovalTeam[]
  grants: PolicyGrant[]
  guardrails: Guardrail[]
  published_by?: string | null
  published_at?: string | null
}

export type RequestApprovalTeam = ApprovalTeam & {
  approved_count: number
  rejected_count: number
}

export type RequestGrant = {
  securable_type: string
  full_name: string
  privilege: string
  fulfillment_status: string
}

export type RequestReview = {
  approval_team: string
  reviewer: string
  decision: "APPROVED" | "REJECTED"
  comment?: string | null
  created_at: string
}

export type AccessRequest = {
  id: string
  template_id: string
  template_version: number
  template_name: string
  approval_strategy: "ANY" | "ALL"
  requested_by: string
  requested_for: string
  routed_for_team: string
  target_securable_type: string
  target_full_name: string
  reason: string
  status: string
  requested_duration_seconds: number
  submitted_at: string
  decided_by?: string | null
  decided_at?: string | null
  decision_comment?: string | null
  fulfilled_at?: string | null
  access_expires_at?: string | null
  approval_teams: RequestApprovalTeam[]
  grants: RequestGrant[]
  guardrails: Array<{
    guardrail_key: string
    name: string
    description?: string | null
    enforcement: string
    configuration: Record<string, unknown>
    evaluation_status: string
  }>
  reviews: RequestReview[]
  events: Array<{
    event_type: string
    actor: string
    comment?: string | null
    created_at: string
  }>
}

export type CreatePolicyTemplateInput = {
  key: string
  name: string
  description?: string
  approval_strategy: "ANY" | "ALL"
  default_duration_seconds?: number
  max_duration_seconds?: number
  affected_teams: string[]
  approval_teams: ApprovalTeam[]
  grants: PolicyGrant[]
}

export type ActionResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: string }
