use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreatePolicyTemplateRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub approval_strategy: String,
    #[serde(default)]
    pub auto_approve: bool,
    pub affected_teams: Vec<String>,
    pub approval_teams: Vec<ApprovalTeam>,
    pub grant: PolicyGrant,
    #[serde(default)]
    pub guardrails: Vec<Guardrail>,
    pub created_by: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApprovalTeam {
    pub team: String,
    pub required_approvals: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PolicyGrant {
    pub securable_type: String,
    pub target_full_name: String,
    pub privilege: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Guardrail {
    pub guardrail_key: String,
    pub name: String,
    pub description: Option<String>,
    pub enforcement: String,
    #[serde(default = "empty_object")]
    pub configuration: Value,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListPolicyTemplatesRequest {
    pub affected_team: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyTemplateSummary {
    pub id: String,
    pub key: String,
    pub version: i32,
    pub name: String,
    pub description: Option<String>,
    pub approval_strategy: String,
    pub auto_approve: bool,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListPolicyTemplatesResponse {
    pub policy_templates: Vec<PolicyTemplateSummary>,
}

#[derive(Debug, Serialize)]
pub struct PolicyTemplate {
    pub id: String,
    pub key: String,
    pub version: i32,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub approval_strategy: String,
    pub auto_approve: bool,
    pub blast_radius_required: bool,
    pub guardrail_review_required: bool,
    pub affected_teams: Vec<String>,
    pub approval_teams: Vec<ApprovalTeam>,
    pub grant: PolicyGrant,
    pub guardrails: Vec<Guardrail>,
    pub created_by: String,
    pub created_at: String,
    pub published_by: Option<String>,
    pub published_at: Option<String>,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}
