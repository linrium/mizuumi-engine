use async_trait::async_trait;
use controlplane_queries::queries::access_requests as queries;
use deadpool_postgres::{Object, Pool};
use tokio_postgres::error::SqlState;

use crate::error::AppError;

use super::dtos::{
    AccessRequest, AccessRequestSummary, CancelAccessRequest, CreateAccessRequest,
    ListAccessRequestsRequest, ListAccessRequestsResponse, RequestApprovalTeam, RequestEvent,
    RequestGrant, RequestGuardrail, RequestReview, ReviewAccessRequest, UpdateAccessRequest,
};

const REQUEST_STATUSES: &[&str] = &[
    "PENDING",
    "ASSESSING",
    "GUARDRAIL_REVIEW",
    "AWAITING_APPROVAL",
    "APPROVED",
    "REJECTED",
    "CANCELLED",
    "FULFILLING",
    "FULFILLED",
    "FAILED",
    "EXPIRED",
];
const SECURABLE_TYPES: &[&str] = &[
    "metastore",
    "catalog",
    "schema",
    "table",
    "function",
    "volume",
    "registered_model",
    "external_location",
    "credential",
];
const DEFAULT_APPROVAL_TEAM: &str = "sovicoSecurity";

fn privileges_for(securable_type: &str) -> &'static [&'static str] {
    match securable_type {
        "metastore" => &["CREATE CATALOG"],
        "catalog" => &["USE CATALOG", "CREATE SCHEMA"],
        "schema" => &[
            "USE SCHEMA",
            "CREATE TABLE",
            "CREATE FUNCTION",
            "CREATE VOLUME",
            "CREATE MODEL",
        ],
        "table" => &["SELECT", "MODIFY"],
        "function" | "registered_model" => &["EXECUTE"],
        "volume" => &["READ VOLUME"],
        "external_location" => &[
            "READ FILES",
            "WRITE FILES",
            "CREATE EXTERNAL TABLE",
            "CREATE EXTERNAL VOLUME",
            "CREATE MANAGED STORAGE",
        ],
        "credential" => &["CREATE EXTERNAL LOCATION", "CREATE STORAGE CREDENTIAL"],
        _ => &[],
    }
}

#[async_trait]
pub trait AccessRequestService: Send + Sync {
    async fn create_access_request(
        &self,
        request: CreateAccessRequest,
    ) -> Result<AccessRequest, AppError>;
    async fn list_access_requests(
        &self,
        request: ListAccessRequestsRequest,
    ) -> Result<ListAccessRequestsResponse, AppError>;
    async fn get_access_request(&self, request_id: String) -> Result<AccessRequest, AppError>;
    async fn approve_access_request(
        &self,
        request_id: String,
        review: ReviewAccessRequest,
    ) -> Result<AccessRequest, AppError>;
    async fn reject_access_request(
        &self,
        request_id: String,
        review: ReviewAccessRequest,
    ) -> Result<AccessRequest, AppError>;
    async fn update_access_request(
        &self,
        request_id: String,
        request: UpdateAccessRequest,
    ) -> Result<AccessRequest, AppError>;
    async fn cancel_access_request(
        &self,
        request_id: String,
        request: CancelAccessRequest,
    ) -> Result<AccessRequest, AppError>;
}

pub struct DefaultAccessRequestService {
    pool: Pool,
}

impl DefaultAccessRequestService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    async fn review_access_request(
        &self,
        request_id: String,
        review: ReviewAccessRequest,
        decision: &'static str,
    ) -> Result<AccessRequest, AppError> {
        require_text("request_id", &request_id)?;
        require_text("approval_team", &review.approval_team)?;
        require_text("reviewer", &review.reviewer)?;
        let comment = review.comment.unwrap_or_default();

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let locked = queries::lock_access_request()
            .bind(&transaction, &request_id)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        if !can_record_review(&locked.status, decision) {
            return Err(AppError::Conflict(format!(
                "access request is {} and cannot be {}",
                locked.status,
                decision.to_ascii_lowercase(),
            )));
        }
        let is_assigned_team = queries::is_access_request_approval_team()
            .bind(&transaction, &request_id, &review.approval_team)
            .one()
            .await?;
        let is_auto_approved_rejection = can_reject_auto_approved_request(
            &locked.status,
            &locked.decided_by,
            &locked.routed_for_team,
            &review.approval_team,
            decision,
        );
        if !is_assigned_team && !is_auto_approved_rejection {
            return Err(AppError::Forbidden(format!(
                "team {} is not assigned to this request",
                review.approval_team
            )));
        }
        if is_auto_approved_rejection {
            queries::create_access_request_approval_team()
                .bind(&transaction, &request_id, &review.approval_team)
                .await?;
        }

        queries::create_access_request_review()
            .bind(
                &transaction,
                &request_id,
                &review.approval_team,
                &review.reviewer,
                &decision,
                &comment,
            )
            .await
            .map_err(map_request_write_error)?;

        let event_type = if decision == "REJECTED" {
            queries::reject_access_request()
                .bind(&transaction, &review.reviewer, &comment, &request_id)
                .await?;
            "REJECTED"
        } else if queries::is_access_request_fully_approved()
            .bind(&transaction, &request_id)
            .one()
            .await?
        {
            queries::approve_access_request()
                .bind(&transaction, &review.reviewer, &comment, &request_id)
                .await?;
            "APPROVED"
        } else {
            "APPROVAL_RECORDED"
        };

        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &event_type,
                &review.reviewer,
                &comment,
            )
            .await?;
        transaction.commit().await?;
        load_access_request(&client, &request_id).await
    }
}

fn can_record_review(status: &str, decision: &str) -> bool {
    status == "AWAITING_APPROVAL" || (status == "APPROVED" && decision == "REJECTED")
}

fn can_cancel_request(status: &str) -> bool {
    status == "AWAITING_APPROVAL"
}

fn can_reject_auto_approved_request(
    status: &str,
    decided_by: &str,
    routed_for_team: &str,
    approval_team: &str,
    decision: &str,
) -> bool {
    status == "APPROVED"
        && decided_by == "system:auto-approve"
        && routed_for_team == approval_team
        && decision == "REJECTED"
}

#[async_trait]
impl AccessRequestService for DefaultAccessRequestService {
    async fn create_access_request(
        &self,
        request: CreateAccessRequest,
    ) -> Result<AccessRequest, AppError> {
        validate_create_request(&request)?;

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let template = queries::resolve_request_policy()
            .bind(
                &transaction,
                &request.routed_for_team,
                &request.target_securable_type,
                &request.target_full_name,
                &request.requested_privilege,
            )
            .opt()
            .await?;
        let template_id = template
            .as_ref()
            .map(|item| item.template_id.as_str())
            .unwrap_or_default();
        let template_version = template
            .as_ref()
            .map(|item| item.version)
            .unwrap_or_default();
        let template_name = template
            .as_ref()
            .map(|item| item.name.as_str())
            .unwrap_or("Default security review");
        let approval_strategy = template
            .as_ref()
            .map(|item| item.approval_strategy.as_str())
            .unwrap_or("ANY");
        let auto_approve = template.as_ref().is_some_and(|item| item.auto_approve);

        let requested_duration = request.requested_duration_seconds;

        let requested_for = request
            .requested_for
            .clone()
            .unwrap_or_else(|| request.requested_by.clone());
        let reason = request.reason.clone();
        let idempotency_key = request.idempotency_key.clone().unwrap_or_default();
        let request_id = queries::create_access_request()
            .bind(
                &transaction,
                &template_id,
                &template_version,
                &template_name,
                &approval_strategy,
                &request.requested_by,
                &requested_for,
                &request.routed_for_team,
                &request.target_securable_type,
                &request.target_full_name,
                &reason,
                &auto_approve,
                &requested_duration,
                &idempotency_key,
            )
            .one()
            .await
            .map_err(map_request_write_error)?;

        if let Some(template) = &template {
            if !template.auto_approve {
                queries::copy_access_request_approval_teams()
                    .bind(
                        &transaction,
                        &request_id,
                        &template.template_id,
                        &template.version,
                    )
                    .await?;
            }
            queries::copy_access_request_grants()
                .bind(
                    &transaction,
                    &request_id,
                    &template.template_id,
                    &template.version,
                )
                .await?;
            queries::copy_access_request_guardrails()
                .bind(
                    &transaction,
                    &request_id,
                    &template.template_id,
                    &template.version,
                )
                .await?;
        } else {
            queries::create_access_request_approval_team()
                .bind(&transaction, &request_id, &DEFAULT_APPROVAL_TEAM)
                .await?;
            queries::create_access_request_grant()
                .bind(
                    &transaction,
                    &request_id,
                    &request.target_securable_type,
                    &request.target_full_name,
                    &request.requested_privilege,
                )
                .await?;
        }

        if !auto_approve
            && queries::count_access_request_approval_teams()
                .bind(&transaction, &request_id)
                .one()
                .await?
                == 0
        {
            return Err(AppError::InvalidParameter(
                "policy template has no approval teams".to_owned(),
            ));
        }
        if queries::count_access_request_grants()
            .bind(&transaction, &request_id)
            .one()
            .await?
            == 0
        {
            return Err(AppError::InvalidParameter(
                "policy template has no grants for the requested securable type".to_owned(),
            ));
        }

        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &"SUBMITTED",
                &request.requested_by,
                &reason,
            )
            .await?;
        if auto_approve {
            queries::create_access_request_event()
                .bind(
                    &transaction,
                    &request_id,
                    &"APPROVED",
                    &"system:auto-approve",
                    &"Automatically approved by policy template",
                )
                .await?;
        }
        transaction.commit().await?;
        load_access_request(&client, &request_id).await
    }

    async fn list_access_requests(
        &self,
        request: ListAccessRequestsRequest,
    ) -> Result<ListAccessRequestsResponse, AppError> {
        if let Some(status) = request.status.as_deref()
            && !REQUEST_STATUSES.contains(&status)
        {
            return Err(AppError::InvalidParameter(format!(
                "unsupported request status: {status}"
            )));
        }
        let client = self.pool.get().await?;
        queries::expire_access_requests().bind(&client).await?;
        let rows = queries::list_access_requests()
            .bind(
                &client,
                &request.status.unwrap_or_default(),
                &request.requested_by.unwrap_or_default(),
                &request.requested_for.unwrap_or_default(),
                &request.approval_team.unwrap_or_default(),
            )
            .all()
            .await?;

        Ok(ListAccessRequestsResponse {
            access_requests: rows.into_iter().map(summary_from_row).collect(),
        })
    }

    async fn get_access_request(&self, request_id: String) -> Result<AccessRequest, AppError> {
        let client = self.pool.get().await?;
        queries::expire_access_requests().bind(&client).await?;
        load_access_request(&client, &request_id).await
    }

    async fn approve_access_request(
        &self,
        request_id: String,
        review: ReviewAccessRequest,
    ) -> Result<AccessRequest, AppError> {
        self.review_access_request(request_id, review, "APPROVED")
            .await
    }

    async fn reject_access_request(
        &self,
        request_id: String,
        review: ReviewAccessRequest,
    ) -> Result<AccessRequest, AppError> {
        self.review_access_request(request_id, review, "REJECTED")
            .await
    }

    async fn update_access_request(
        &self,
        request_id: String,
        request: UpdateAccessRequest,
    ) -> Result<AccessRequest, AppError> {
        require_text("request_id", &request_id)?;
        require_text("requested_by", &request.requested_by)?;
        require_text("reason", &request.reason)?;
        validate_duration(request.requested_duration_seconds)?;

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let status = queries::lock_owned_access_request()
            .bind(&transaction, &request_id, &request.requested_by)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        if status != "AWAITING_APPROVAL" {
            return Err(AppError::Conflict(format!(
                "access request is {status}, not editable"
            )));
        }

        queries::clear_access_request_reviews()
            .bind(&transaction, &request_id)
            .await?;
        queries::update_access_request()
            .bind(
                &transaction,
                &request.reason,
                &request.requested_duration_seconds,
                &request_id,
            )
            .await?;
        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &"UPDATED",
                &request.requested_by,
                &"Request details updated; previous reviews cleared",
            )
            .await?;
        transaction.commit().await?;
        load_access_request(&client, &request_id).await
    }

    async fn cancel_access_request(
        &self,
        request_id: String,
        request: CancelAccessRequest,
    ) -> Result<AccessRequest, AppError> {
        require_text("request_id", &request_id)?;
        require_text("requested_by", &request.requested_by)?;

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let status = queries::lock_owned_access_request()
            .bind(&transaction, &request_id, &request.requested_by)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        if !can_cancel_request(&status) {
            return Err(AppError::Conflict(format!(
                "access request is {status}, not cancellable"
            )));
        }

        queries::cancel_access_request()
            .bind(&transaction, &request.requested_by, &request_id)
            .await?;
        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &"CANCELLED",
                &request.requested_by,
                &"Request cancelled by requester",
            )
            .await?;
        transaction.commit().await?;
        load_access_request(&client, &request_id).await
    }
}

async fn load_access_request(client: &Object, request_id: &str) -> Result<AccessRequest, AppError> {
    let row = queries::get_access_request()
        .bind(client, &request_id)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
    let approval_teams = queries::list_access_request_approval_teams()
        .bind(client, &request_id)
        .all()
        .await?
        .into_iter()
        .map(|team| RequestApprovalTeam {
            team: team.team,
            required_approvals: team.required_approvals,
            approved_count: team.approved_count,
            rejected_count: team.rejected_count,
        })
        .collect();
    let grants = queries::list_access_request_grants()
        .bind(client, &request_id)
        .all()
        .await?
        .into_iter()
        .map(|grant| RequestGrant {
            securable_type: grant.securable_type,
            full_name: grant.full_name,
            privilege: grant.privilege,
            fulfillment_status: grant.fulfillment_status,
        })
        .collect();
    let guardrails = queries::list_access_request_guardrails()
        .bind(client, &request_id)
        .all()
        .await?
        .into_iter()
        .map(|guardrail| RequestGuardrail {
            guardrail_key: guardrail.guardrail_key,
            name: guardrail.name,
            description: non_empty(guardrail.description),
            enforcement: guardrail.enforcement,
            configuration: guardrail.configuration,
            evaluation_status: guardrail.evaluation_status,
        })
        .collect();
    let reviews = queries::list_access_request_reviews()
        .bind(client, &request_id)
        .all()
        .await?
        .into_iter()
        .map(|review| RequestReview {
            approval_team: review.approval_team,
            reviewer: review.reviewer,
            decision: review.decision,
            comment: non_empty(review.comment),
            created_at: review.created_at,
        })
        .collect();
    let events = queries::list_access_request_events()
        .bind(client, &request_id)
        .all()
        .await?
        .into_iter()
        .map(|event| RequestEvent {
            event_type: event.event_type,
            actor: event.actor,
            comment: non_empty(event.comment),
            created_at: event.created_at,
        })
        .collect();

    Ok(AccessRequest {
        id: row.id,
        template_id: row.template_id,
        template_version: row.template_version,
        template_name: row.template_name,
        approval_strategy: row.approval_strategy,
        requested_by: row.requested_by,
        requested_for: row.requested_for,
        routed_for_team: row.routed_for_team,
        target_securable_type: row.target_securable_type,
        target_full_name: row.target_full_name,
        reason: row.reason,
        status: row.status,
        requested_duration_seconds: row.requested_duration_seconds,
        submitted_at: row.submitted_at,
        decided_by: non_empty(row.decided_by),
        decided_at: non_empty(row.decided_at),
        decision_comment: non_empty(row.decision_comment),
        fulfilled_at: non_empty(row.fulfilled_at),
        access_expires_at: non_empty(row.access_expires_at),
        approval_teams,
        grants,
        guardrails,
        reviews,
        events,
    })
}

fn summary_from_row(row: queries::ListAccessRequests) -> AccessRequestSummary {
    AccessRequestSummary {
        id: row.id,
        template_id: row.template_id,
        template_version: row.template_version,
        template_name: row.template_name,
        requested_by: row.requested_by,
        requested_for: row.requested_for,
        routed_for_team: row.routed_for_team,
        target_securable_type: row.target_securable_type,
        target_full_name: row.target_full_name,
        reason: row.reason,
        status: row.status,
        requested_duration_seconds: row.requested_duration_seconds,
        submitted_at: row.submitted_at,
        decided_by: non_empty(row.decided_by),
        decided_at: non_empty(row.decided_at),
        decision_comment: non_empty(row.decision_comment),
    }
}

fn validate_create_request(request: &CreateAccessRequest) -> Result<(), AppError> {
    require_text("requested_by", &request.requested_by)?;
    if let Some(requested_for) = request.requested_for.as_deref() {
        require_text("requested_for", requested_for)?;
    }
    require_text("routed_for_team", &request.routed_for_team)?;
    require_text("target_full_name", &request.target_full_name)?;
    require_text("requested_privilege", &request.requested_privilege)?;
    require_text("reason", &request.reason)?;
    if !SECURABLE_TYPES.contains(&request.target_securable_type.as_str()) {
        return Err(AppError::InvalidParameter(format!(
            "unsupported target_securable_type: {}",
            request.target_securable_type
        )));
    }
    if !privileges_for(&request.target_securable_type)
        .contains(&request.requested_privilege.as_str())
    {
        return Err(AppError::InvalidParameter(format!(
            "privilege {} is not valid for target_securable_type {}",
            request.requested_privilege, request.target_securable_type
        )));
    }
    validate_duration(request.requested_duration_seconds)?;
    if let Some(key) = request.idempotency_key.as_deref() {
        require_text("idempotency_key", key)?;
    }
    Ok(())
}

fn validate_duration(duration: i64) -> Result<(), AppError> {
    if duration <= 0 || duration % 3600 != 0 {
        return Err(AppError::InvalidParameter(
            "requested_duration_seconds must be a positive whole number of hours".to_owned(),
        ));
    }
    Ok(())
}

fn require_text(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::InvalidParameter(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn map_request_write_error(error: tokio_postgres::Error) -> AppError {
    match error.code() {
        Some(code) if code == &SqlState::UNIQUE_VIOLATION => {
            AppError::Conflict("request or review already exists".to_owned())
        }
        Some(code) if code == &SqlState::CHECK_VIOLATION => {
            AppError::InvalidParameter("request violates a constraint".to_owned())
        }
        _ => AppError::Postgres(error),
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_request_can_be_rejected_but_not_approved_again() {
        assert!(can_record_review("APPROVED", "REJECTED"));
        assert!(!can_record_review("APPROVED", "APPROVED"));
    }

    #[test]
    fn awaiting_request_accepts_either_review_decision() {
        assert!(can_record_review("AWAITING_APPROVAL", "APPROVED"));
        assert!(can_record_review("AWAITING_APPROVAL", "REJECTED"));
    }

    #[test]
    fn only_awaiting_request_can_be_cancelled() {
        assert!(can_cancel_request("AWAITING_APPROVAL"));
        assert!(!can_cancel_request("APPROVED"));
        assert!(!can_cancel_request("FULFILLED"));
        assert!(!can_cancel_request("REJECTED"));
    }

    #[test]
    fn auto_approved_request_can_be_rejected_by_its_routed_team() {
        assert!(can_reject_auto_approved_request(
            "APPROVED",
            "system:auto-approve",
            "analytics",
            "analytics",
            "REJECTED",
        ));
        assert!(!can_reject_auto_approved_request(
            "APPROVED",
            "system:auto-approve",
            "analytics",
            "security",
            "REJECTED",
        ));
    }

    fn valid_request() -> CreateAccessRequest {
        CreateAccessRequest {
            requested_by: "alice".to_owned(),
            requested_for: None,
            routed_for_team: "analytics".to_owned(),
            target_securable_type: "table".to_owned(),
            target_full_name: "main.analytics.orders".to_owned(),
            requested_privilege: "SELECT".to_owned(),
            reason: "Investigate a customer incident".to_owned(),
            requested_duration_seconds: 3600,
            idempotency_key: None,
        }
    }

    #[test]
    fn create_request_rejects_blank_reason() {
        let mut request = valid_request();
        request.reason = "  ".to_owned();

        let error = validate_create_request(&request).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid parameter: reason must not be empty"
        );
    }

    #[test]
    fn create_request_requires_positive_whole_hours() {
        for duration in [-3600, 0, 1, 3599, 3601] {
            let mut request = valid_request();
            request.requested_duration_seconds = duration;

            let error = validate_create_request(&request).unwrap_err();
            assert_eq!(
                error.to_string(),
                "invalid parameter: requested_duration_seconds must be a positive whole number of hours"
            );
        }
    }

    #[test]
    fn create_request_rejects_privilege_for_another_securable_type() {
        let mut request = valid_request();
        request.requested_privilege = "USE CATALOG".to_owned();

        let error = validate_create_request(&request).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid parameter: privilege USE CATALOG is not valid for target_securable_type table"
        );
    }
}
