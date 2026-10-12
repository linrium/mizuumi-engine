use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreateAccessRequest {
    pub requested_by: String,
    pub requested_for: Option<String>,
    pub routed_for_team: String,
    pub target_securable_type: String,
    pub target_full_name: String,
    pub requested_privilege: String,
    pub reason: String,
    pub requested_duration_seconds: i64,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListAccessRequestsRequest {
    pub status: Option<String>,
    pub approval_team: Option<String>,
    pub requested_by: Option<String>,
    pub requested_for: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewAccessRequest {
    pub approval_team: String,
    pub reviewer: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccessRequest {
    pub requested_by: String,
    pub reason: String,
    pub requested_duration_seconds: i64,
}

#[derive(Debug, Deserialize)]
pub struct RevokeAccessRequest {
    pub requested_by: String,
}

#[derive(Debug, Serialize)]
pub struct ListAccessRequestsResponse {
    pub access_requests: Vec<AccessRequestSummary>,
}

#[derive(Debug, Serialize)]
pub struct AccessRequestSummary {
    pub id: String,
    pub template_id: String,
    pub template_version: i32,
    pub template_name: String,
    pub requested_by: String,
    pub requested_for: String,
    pub routed_for_team: String,
    pub target_securable_type: String,
    pub target_full_name: String,
    pub reason: String,
    pub status: String,
    pub requested_duration_seconds: i64,
    pub submitted_at: String,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub decision_comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccessRequest {
    pub id: String,
    pub template_id: String,
    pub template_version: i32,
    pub template_name: String,
    pub approval_strategy: String,
    pub requested_by: String,
    pub requested_for: String,
    pub routed_for_team: String,
    pub target_securable_type: String,
    pub target_full_name: String,
    pub reason: String,
    pub status: String,
    pub requested_duration_seconds: i64,
    pub submitted_at: String,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub decision_comment: Option<String>,
    pub fulfilled_at: Option<String>,
    pub access_expires_at: Option<String>,
    pub approval_teams: Vec<RequestApprovalTeam>,
    pub grants: Vec<RequestGrant>,
    pub guardrails: Vec<RequestGuardrail>,
    pub reviews: Vec<RequestReview>,
    pub events: Vec<RequestEvent>,
}

#[derive(Debug, Serialize)]
pub struct RequestApprovalTeam {
    pub team: String,
    pub required_approvals: i32,
    pub approved_count: i32,
    pub rejected_count: i32,
}

#[derive(Debug, Serialize)]
pub struct RequestGrant {
    pub securable_type: String,
    pub full_name: String,
    pub privilege: String,
    pub fulfillment_status: String,
}

#[derive(Debug, Serialize)]
pub struct RequestGuardrail {
    pub guardrail_key: String,
    pub name: String,
    pub description: Option<String>,
    pub enforcement: String,
    pub configuration: Value,
    pub evaluation_status: String,
}

#[derive(Debug, Serialize)]
pub struct RequestReview {
    pub approval_team: String,
    pub reviewer: String,
    pub decision: String,
    pub comment: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct RequestEvent {
    pub event_type: String,
    pub actor: String,
    pub comment: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::CreateAccessRequest;

    fn valid_request() -> serde_json::Value {
        json!({
            "requested_by": "alice",
            "routed_for_team": "analytics",
            "target_securable_type": "table",
            "target_full_name": "main.analytics.orders",
            "requested_privilege": "SELECT",
            "reason": "Investigate a customer incident",
            "requested_duration_seconds": 3600
        })
    }

    #[test]
    fn create_request_requires_reason() {
        let mut request = valid_request();
        request.as_object_mut().unwrap().remove("reason");

        assert!(serde_json::from_value::<CreateAccessRequest>(request).is_err());
    }

    #[test]
    fn create_request_requires_duration() {
        let mut request = valid_request();
        request
            .as_object_mut()
            .unwrap()
            .remove("requested_duration_seconds");

        assert!(serde_json::from_value::<CreateAccessRequest>(request).is_err());
    }
}
