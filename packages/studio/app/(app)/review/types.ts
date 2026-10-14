export type ApprovalTeam = {
  team: string
  required_approvals: number
}

export type PolicyGrant = {
  securable_type: string
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
  auto_approve: boolean
  created_by: string
  created_at: string
}

export type PolicyTemplate = PolicyTemplateSummary & {
  status: "DRAFT" | "PUBLISHED" | "RETIRED"
  blast_radius_required: boolean
  guardrail_review_required: boolean
  affected_teams: string[]
  approval_teams: ApprovalTeam[]
  grant: PolicyGrant
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

export type ChronicleAccessRequestEvidence = {
  index: number
  action: string
  principal?: string | null
  status: string
  finished_at: string
  verified: boolean
  checkpoint_verified: boolean
  verification_error?: string | null
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
  contract?: {
    id: string
    version: string
    document_hash: string
    evaluation_status: "PASSED" | "WARNING"
    evaluated_at: string
    snapshot: Record<string, unknown>
    evaluations: ContractEvaluation[]
  } | null
  purpose_code?: string | null
  consumer_context: {
    consumer_domain?: string
    processing_countries?: string[]
    cross_border_transfer?: boolean
    ai_use?: boolean
    intended_output?: string | null
  }
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

export type ContractEvaluation = {
  rule_key: string
  name: string
  enforcement: "BLOCK" | "ADVISORY"
  status: "PASS" | "WARN"
  message: string
  evidence: Record<string, unknown>
}

export type CreatePolicyTemplateInput = {
  key: string
  name: string
  description?: string
  approval_strategy: "ANY" | "ALL"
  auto_approve: boolean
  affected_teams: string[]
  approval_teams: ApprovalTeam[]
  grant: PolicyGrant
}

export type CreateAccessRequestInput = {
  routed_for_team: string
  target_securable_type: string
  target_full_name: string
  requested_privilege: string
  purpose_code?: string
  consumer_context?: {
    consumer_domain: string
    processing_countries: string[]
    cross_border_transfer: boolean
    ai_use: boolean
    intended_output?: string
  }
  acknowledged_contract_terms?: boolean
  reason: string
  duration_hours: number
}

export type ActionResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: string }
