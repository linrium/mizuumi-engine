use std::{sync::Arc, time::SystemTime};

use async_trait::async_trait;
use controlplane_queries::queries::access_requests as queries;
use deadpool_postgres::{Object, Pool};
use serde::Serialize;
use serde_json::{Value, json};
use tokio_postgres::error::SqlState;

use crate::error::AppError;
use crate::infrastructure::unity_catalog::UnityCatalog;

use super::blast_radius::{BlastRadiusEvaluator, BlastRadiusInput};
use super::dtos::{
    AccessRequest, AccessRequestSummary, CancelAccessRequest, ConsumerContext, CreateAccessRequest,
    ListAccessRequestsRequest, ListAccessRequestsResponse, RequestApprovalTeam,
    RequestContractBinding, RequestEvent, RequestGrant, RequestGuardrail, RequestReview,
    ReviewAccessRequest, UpdateAccessRequest,
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
const PURPOSE_CODES: &[&str] = &[
    "ANALYTICS",
    "DATA_PRODUCT",
    "OPERATIONS",
    "COMPLIANCE",
    "AI_MODEL",
];

fn record_access_request_event(request_id: &str, event_type: &str, actor: &str) {
    let recorded_at = SystemTime::now();
    chronicle_audit::record(chronicle_audit::AuditEvent {
        action: format!("access_requests.{}", event_type.to_ascii_lowercase()),
        principal: Some(actor.to_owned()),
        fields: vec![
            ("request_id".to_owned(), request_id.to_owned()),
            ("event_type".to_owned(), event_type.to_owned()),
            ("actor".to_owned(), actor.to_owned()),
        ],
        started_at: recorded_at,
        finished_at: recorded_at,
        status: "ok".to_owned(),
        error: None,
    });
}

#[derive(Debug, Serialize)]
struct ContractEvaluation {
    rule_key: &'static str,
    name: &'static str,
    enforcement: &'static str,
    status: &'static str,
    message: String,
    evidence: Value,
}

struct EvaluatedContract {
    id: String,
    version: String,
    document_hash: String,
    snapshot: Value,
    evaluations: Value,
    status: &'static str,
}

fn privileges_for(securable_type: &str) -> &'static [&'static str] {
    match securable_type {
        "metastore" => &["CREATE CATALOG"],
        "catalog" => &["BROWSE", "USE CATALOG", "CREATE SCHEMA"],
        "schema" => &[
            "BROWSE",
            "USE SCHEMA",
            "CREATE TABLE",
            "CREATE FUNCTION",
            "CREATE VOLUME",
            "CREATE MODEL",
        ],
        "table" => &["BROWSE", "SELECT", "MODIFY"],
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

fn needs_revocation(fulfillment_status: &str) -> bool {
    matches!(fulfillment_status, "GRANTING" | "GRANTED" | "UNCHANGED")
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
    unity_catalog: Arc<dyn UnityCatalog>,
    blast_radius: Arc<dyn BlastRadiusEvaluator>,
}

impl DefaultAccessRequestService {
    pub fn new(
        pool: Pool,
        unity_catalog: Arc<dyn UnityCatalog>,
        blast_radius: Arc<dyn BlastRadiusEvaluator>,
    ) -> Self {
        Self {
            pool,
            unity_catalog,
            blast_radius,
        }
    }

    async fn fulfill_access_request(&self, request_id: &str) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        if let Some(state) = queries::get_access_request_contract_state()
            .bind(&client, &request_id)
            .opt()
            .await?
        {
            let validation_error = contract_binding_error(
                &state.active_version,
                &state.active_document_hash,
                &state.sync_state,
                &state.bound_version,
                &state.bound_document_hash,
            );
            if let Some(message) = validation_error {
                queries::fail_access_request_contract_validation()
                    .bind(&client, &message, &request_id)
                    .await?;
                queries::create_access_request_event()
                    .bind(
                        &client,
                        &request_id,
                        &"CONTRACT_VALIDATION_FAILED",
                        &"system:contract-evaluation",
                        &message,
                    )
                    .await?;
                record_access_request_event(
                    request_id,
                    "CONTRACT_VALIDATION_FAILED",
                    "system:contract-evaluation",
                );
                return Ok(());
            }
        }
        let requested_for = match queries::start_access_request_fulfillment()
            .bind(&client, &request_id)
            .opt()
            .await?
        {
            Some(principal) => principal,
            None => return Ok(()),
        };
        queries::create_access_request_event()
            .bind(
                &client,
                &request_id,
                &"FULFILLMENT_STARTED",
                &"system:unity-catalog",
                &"Applying approved grants to Unity Catalog",
            )
            .await?;
        record_access_request_event(request_id, "FULFILLMENT_STARTED", "system:unity-catalog");

        let grants = queries::list_access_request_grants()
            .bind(&client, &request_id)
            .all()
            .await?;
        let mut has_managed_grants = false;
        for grant in &grants {
            let managed_by_another_request = queries::has_other_active_managed_grant()
                .bind(
                    &client,
                    &request_id,
                    &requested_for,
                    &grant.securable_type,
                    &grant.full_name,
                    &grant.privilege,
                )
                .one()
                .await?;
            let result = if managed_by_another_request {
                Ok("GRANTED")
            } else {
                match self
                    .unity_catalog
                    .has_privilege(
                        &grant.securable_type,
                        &grant.full_name,
                        &requested_for,
                        &grant.privilege,
                    )
                    .await
                {
                    Ok(true) => Ok("UNCHANGED"),
                    Ok(false) => {
                        queries::set_access_request_grant_status()
                            .bind(
                                &client,
                                &"GRANTING",
                                &"",
                                &request_id,
                                &grant.securable_type,
                                &grant.full_name,
                                &grant.privilege,
                            )
                            .await?;
                        has_managed_grants = true;
                        self.unity_catalog
                            .grant(
                                &grant.securable_type,
                                &grant.full_name,
                                &requested_for,
                                &grant.privilege,
                            )
                            .await
                            .map(|()| "GRANTED")
                    }
                    Err(error) => Err(error),
                }
            };

            match result {
                Ok(status) => {
                    queries::set_access_request_grant_status()
                        .bind(
                            &client,
                            &status,
                            &"",
                            &request_id,
                            &grant.securable_type,
                            &grant.full_name,
                            &grant.privilege,
                        )
                        .await?;
                    has_managed_grants |= status == "GRANTED";
                }
                Err(error) => {
                    let failure_message = error.to_string();
                    queries::fail_pending_access_request_grants()
                        .bind(&client, &failure_message, &request_id)
                        .await?;
                    queries::fail_access_request_fulfillment()
                        .bind(&client, &request_id)
                        .await?;
                    queries::create_access_request_event()
                        .bind(
                            &client,
                            &request_id,
                            &"FULFILLMENT_FAILED",
                            &"system:unity-catalog",
                            &failure_message,
                        )
                        .await?;
                    record_access_request_event(
                        request_id,
                        "FULFILLMENT_FAILED",
                        "system:unity-catalog",
                    );
                    tracing::error!(
                        %request_id,
                        error = %error,
                        "failed to fulfill access request in Unity Catalog"
                    );
                    if has_managed_grants {
                        drop(client);
                        if let Err(rollback_error) = self
                            .revoke_access_request_grants(request_id, &requested_for)
                            .await
                        {
                            tracing::error!(
                                %request_id,
                                error = %rollback_error,
                                "failed to roll back a partially fulfilled Unity Catalog request; reconciliation will retry"
                            );
                        }
                    }
                    return Ok(());
                }
            }
        }

        queries::complete_access_request_fulfillment()
            .bind(&client, &request_id)
            .await?;
        queries::create_access_request_event()
            .bind(
                &client,
                &request_id,
                &"FULFILLMENT_SUCCEEDED",
                &"system:unity-catalog",
                &"Approved grants applied to Unity Catalog",
            )
            .await?;
        record_access_request_event(request_id, "FULFILLMENT_SUCCEEDED", "system:unity-catalog");
        Ok(())
    }

    async fn reconcile_access_requests(&self) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        let expired_unfulfilled = queries::expire_unfulfilled_access_requests()
            .bind(&client)
            .all()
            .await?;
        for request_id in expired_unfulfilled {
            record_access_request_event(&request_id, "EXPIRED", "system:expiry");
        }
        let approved_ids = queries::list_approved_access_request_ids()
            .bind(&client)
            .all()
            .await?;
        drop(client);

        for request_id in approved_ids {
            self.fulfill_access_request(&request_id).await?;
        }

        let client = self.pool.get().await?;
        let expired = queries::list_expired_fulfilled_access_requests()
            .bind(&client)
            .all()
            .await?;
        drop(client);

        for request in expired {
            if let Err(error) = self
                .expire_access_request(&request.id, &request.requested_for)
                .await
            {
                tracing::error!(
                    request_id = %request.id,
                    %error,
                    "failed to revoke an expired Unity Catalog access request"
                );
            }
        }

        let client = self.pool.get().await?;
        let pending_revocations = queries::list_access_requests_pending_revocation()
            .bind(&client)
            .all()
            .await?;
        drop(client);
        for request in pending_revocations {
            if let Err(error) = self
                .revoke_access_request_grants(&request.id, &request.requested_for)
                .await
            {
                tracing::error!(
                    request_id = %request.id,
                    %error,
                    "failed to retry Unity Catalog revocation for a rejected request"
                );
            }
        }
        Ok(())
    }

    async fn revoke_access_request_grants(
        &self,
        request_id: &str,
        requested_for: &str,
    ) -> Result<(), AppError> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        queries::lock_access_request()
            .bind(&transaction, &request_id)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        let grants = queries::list_access_request_grants()
            .bind(&transaction, &request_id)
            .all()
            .await?;
        let has_pending_revocation = grants
            .iter()
            .any(|grant| needs_revocation(&grant.fulfillment_status));
        if !has_pending_revocation {
            transaction.commit().await?;
            return Ok(());
        }

        for grant in &grants {
            if matches!(grant.fulfillment_status.as_str(), "GRANTING" | "GRANTED") {
                let still_required = queries::has_other_active_managed_grant()
                    .bind(
                        &transaction,
                        &request_id,
                        &requested_for,
                        &grant.securable_type,
                        &grant.full_name,
                        &grant.privilege,
                    )
                    .one()
                    .await?;
                if !still_required {
                    self.unity_catalog
                        .revoke(
                            &grant.securable_type,
                            &grant.full_name,
                            requested_for,
                            &grant.privilege,
                        )
                        .await
                        .map_err(|error| AppError::UnityCatalog(error.to_string()))?;
                }
            }
        }

        for grant in &grants {
            queries::set_access_request_grant_status()
                .bind(
                    &transaction,
                    &"REVOKED",
                    &"",
                    &request_id,
                    &grant.securable_type,
                    &grant.full_name,
                    &grant.privilege,
                )
                .await?;
        }
        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &"ACCESS_REVOKED",
                &"system:unity-catalog",
                &"Control-plane-managed Unity Catalog grants revoked",
            )
            .await?;
        transaction.commit().await?;
        record_access_request_event(request_id, "ACCESS_REVOKED", "system:unity-catalog");
        Ok(())
    }

    async fn expire_access_request(
        &self,
        request_id: &str,
        requested_for: &str,
    ) -> Result<(), AppError> {
        self.revoke_access_request_grants(request_id, requested_for)
            .await?;
        let client = self.pool.get().await?;
        let expired_request_id = queries::expire_access_request()
            .bind(&client, &request_id)
            .opt()
            .await?;
        if let Some(expired_request_id) = expired_request_id {
            record_access_request_event(&expired_request_id, "EXPIRED", "system:expiry");
        }
        Ok(())
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
        let revoke_fulfilled_grants = locked.status == "FULFILLED" && decision == "REJECTED";
        let requested_for = locked.requested_for.clone();
        if !is_assigned_team && !is_auto_approved_rejection {
            return Err(AppError::Forbidden(format!(
                "team {} is not assigned to this request",
                review.approval_team
            )));
        }
        if decision == "APPROVED"
            && let Some(state) = queries::get_access_request_contract_state()
                .bind(&transaction, &request_id)
                .opt()
                .await?
            && let Some(message) = contract_binding_error(
                &state.active_version,
                &state.active_document_hash,
                &state.sync_state,
                &state.bound_version,
                &state.bound_document_hash,
            )
        {
            return Err(AppError::Conflict(message));
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
        record_access_request_event(&request_id, event_type, &review.reviewer);
        drop(client);
        if event_type == "APPROVED" {
            self.fulfill_access_request(&request_id).await?;
        } else if revoke_fulfilled_grants {
            if let Err(error) = self
                .revoke_access_request_grants(&request_id, &requested_for)
                .await
            {
                tracing::error!(
                    %request_id,
                    %error,
                    "failed to revoke Unity Catalog grants after rejection; reconciliation will retry"
                );
            }
        }
        let client = self.pool.get().await?;
        load_access_request(&client, &request_id).await
    }
}

fn can_record_review(status: &str, decision: &str) -> bool {
    status == "AWAITING_APPROVAL"
        || (matches!(status, "APPROVED" | "FULFILLED") && decision == "REJECTED")
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
    matches!(status, "APPROVED" | "FULFILLED")
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
        let template = queries::resolve_request_policy()
            .bind(
                &client,
                &request.routed_for_team,
                &request.target_securable_type,
                &request.target_full_name,
                &request.requested_privilege,
            )
            .opt()
            .await?;
        let evaluated_contract = if request.target_securable_type == "table" {
            let (catalog_name, schema_name, table_name) =
                split_table_name(&request.target_full_name)?;
            let contract = queries::get_active_contract_for_access_request()
                .bind(&client, &catalog_name, &schema_name, &table_name)
                .opt()
                .await?
                .ok_or_else(|| {
                    AppError::Conflict(format!(
                        "table {} has no active data contract",
                        request.target_full_name
                    ))
                })?;
            Some(evaluate_contract(
                contract.id,
                contract.version,
                contract.document,
                contract.document_hash,
                contract.sync_state,
                contract.latest_quality_status,
                request
                    .consumer_context
                    .as_ref()
                    .expect("validated context"),
                request.requested_duration_seconds,
            )?)
        } else {
            None
        };
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
        let auto_approve = template.as_ref().is_some_and(|item| item.auto_approve)
            && evaluated_contract
                .as_ref()
                .is_none_or(|contract| contract.status == "PASSED");

        let requested_duration = request.requested_duration_seconds;

        let requested_for = request
            .requested_for
            .clone()
            .unwrap_or_else(|| request.requested_by.clone());
        let reason = request.reason.clone();
        let idempotency_key = request.idempotency_key.clone().unwrap_or_default();
        let consumer_context = request
            .consumer_context
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?
            .unwrap_or_else(|| json!({}));
        let contract_id = evaluated_contract
            .as_ref()
            .map(|contract| contract.id.as_str())
            .unwrap_or_default();
        let contract_version = evaluated_contract
            .as_ref()
            .map(|contract| contract.version.as_str())
            .unwrap_or_default();
        let contract_document_hash = evaluated_contract
            .as_ref()
            .map(|contract| contract.document_hash.as_str())
            .unwrap_or_default();
        let contract_snapshot = evaluated_contract
            .as_ref()
            .map(|contract| contract.snapshot.clone())
            .unwrap_or_else(|| json!({}));
        let contract_evaluations = evaluated_contract
            .as_ref()
            .map(|contract| contract.evaluations.clone())
            .unwrap_or_else(|| json!([]));
        let contract_evaluation_status = evaluated_contract
            .as_ref()
            .map(|contract| contract.status)
            .unwrap_or("NOT_APPLICABLE");
        let blast_radius = self
            .blast_radius
            .evaluate(BlastRadiusInput {
                request: &request,
                contract: evaluated_contract
                    .as_ref()
                    .map(|contract| &contract.snapshot),
                contract_evaluations: evaluated_contract
                    .as_ref()
                    .map(|contract| &contract.evaluations),
            })
            .await
            .map_err(|error| AppError::BlastRadius(error.to_string()))?;
        let blast_radius_json = serde_json::to_value(&blast_radius)?;
        let purpose_code = request.purpose_code.as_deref().unwrap_or_default();
        let transaction = client.transaction().await?;
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
                &contract_id,
                &contract_version,
                &contract_document_hash,
                &purpose_code,
                &consumer_context,
                &contract_snapshot,
                &contract_evaluations,
                &contract_evaluation_status,
                &blast_radius_json,
                &reason,
                &auto_approve,
                &requested_duration,
                &idempotency_key,
            )
            .one()
            .await
            .map_err(map_request_write_error)?;

        if let Some(template) = &template {
            if !auto_approve {
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
            queries::create_access_request_approval_team()
                .bind(&transaction, &request_id, &DEFAULT_APPROVAL_TEAM)
                .await?;
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
        if let Some(contract) = &evaluated_contract {
            queries::create_access_request_event()
                .bind(
                    &transaction,
                    &request_id,
                    &"CONTRACT_BOUND",
                    &request.requested_by,
                    &format!(
                        "Bound data contract {} version {} ({})",
                        contract.id, contract.version, contract.status
                    ),
                )
                .await?;
            queries::create_access_request_event()
                .bind(
                    &transaction,
                    &request_id,
                    &"CONTRACT_TERMS_ACKNOWLEDGED",
                    &request.requested_by,
                    &"Requester acknowledged the frozen data contract terms",
                )
                .await?;
        }
        let blast_radius_event = if blast_radius.status == "COMPLETED" {
            "BLAST_RADIUS_EVALUATED"
        } else {
            "BLAST_RADIUS_SKIPPED"
        };
        queries::create_access_request_event()
            .bind(
                &transaction,
                &request_id,
                &blast_radius_event,
                &"system:blast-radius-agent",
                &format!(
                    "{} risk (score {}), recommendation {}: {}",
                    blast_radius.risk_level,
                    blast_radius.risk_score,
                    blast_radius.recommendation,
                    blast_radius.summary
                ),
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
        record_access_request_event(&request_id, "SUBMITTED", &request.requested_by);
        if evaluated_contract.is_some() {
            record_access_request_event(&request_id, "CONTRACT_BOUND", &request.requested_by);
            record_access_request_event(
                &request_id,
                "CONTRACT_TERMS_ACKNOWLEDGED",
                &request.requested_by,
            );
        }
        record_access_request_event(&request_id, blast_radius_event, "system:blast-radius-agent");
        if auto_approve {
            record_access_request_event(&request_id, "APPROVED", "system:auto-approve");
        }
        drop(client);
        if auto_approve {
            self.fulfill_access_request(&request_id).await?;
        }
        let client = self.pool.get().await?;
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
        self.reconcile_access_requests().await?;
        let client = self.pool.get().await?;
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
        self.reconcile_access_requests().await?;
        let client = self.pool.get().await?;
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
        let locked = queries::lock_owned_access_request()
            .bind(&transaction, &request_id, &request.requested_by)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        if locked.status != "AWAITING_APPROVAL" {
            return Err(AppError::Conflict(format!(
                "access request is {}, not editable",
                locked.status
            )));
        }
        if let Some(max_hours) =
            custom_property(&locked.contract_document, "maxAccessDurationHours")
                .and_then(Value::as_i64)
            && request.requested_duration_seconds > max_hours * 3600
        {
            return Err(AppError::Conflict(format!(
                "requested duration exceeds the contract maximum of {max_hours} hours"
            )));
        }
        if let Some(state) = queries::get_access_request_contract_state()
            .bind(&transaction, &request_id)
            .opt()
            .await?
            && let Some(message) = contract_binding_error(
                &state.active_version,
                &state.active_document_hash,
                &state.sync_state,
                &state.bound_version,
                &state.bound_document_hash,
            )
        {
            return Err(AppError::Conflict(message));
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
        record_access_request_event(&request_id, "UPDATED", &request.requested_by);
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
        let locked = queries::lock_owned_access_request()
            .bind(&transaction, &request_id, &request.requested_by)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("access request {request_id}")))?;
        if !can_cancel_request(&locked.status) {
            return Err(AppError::Conflict(format!(
                "access request is {}, not cancellable",
                locked.status
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
        record_access_request_event(&request_id, "CANCELLED", &request.requested_by);
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
        contract: (!row.contract_id.is_empty()).then(|| RequestContractBinding {
            id: row.contract_id,
            version: row.contract_version,
            document_hash: row.contract_document_hash,
            evaluation_status: row.contract_evaluation_status,
            evaluated_at: row.contract_evaluated_at,
            snapshot: row.contract_snapshot,
            evaluations: row.contract_evaluations,
        }),
        blast_radius: row.blast_radius,
        blast_radius_evaluated_at: non_empty(row.blast_radius_evaluated_at),
        purpose_code: non_empty(row.purpose_code),
        consumer_context: row.consumer_context,
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
        blast_radius: row.blast_radius,
        blast_radius_evaluated_at: non_empty(row.blast_radius_evaluated_at),
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
    if request.target_securable_type == "table" {
        let purpose_code = request.purpose_code.as_deref().ok_or_else(|| {
            AppError::InvalidParameter("purpose_code is required for table access".to_owned())
        })?;
        if !PURPOSE_CODES.contains(&purpose_code) {
            return Err(AppError::InvalidParameter(format!(
                "unsupported purpose_code: {purpose_code}"
            )));
        }
        let context = request.consumer_context.as_ref().ok_or_else(|| {
            AppError::InvalidParameter("consumer_context is required for table access".to_owned())
        })?;
        require_text("consumer_context.consumer_domain", &context.consumer_domain)?;
        if context.processing_countries.is_empty()
            || context
                .processing_countries
                .iter()
                .any(|country| country.trim().is_empty())
        {
            return Err(AppError::InvalidParameter(
                "consumer_context.processing_countries must contain at least one country"
                    .to_owned(),
            ));
        }
        if !request.acknowledged_contract_terms {
            return Err(AppError::InvalidParameter(
                "acknowledged_contract_terms must be true for table access".to_owned(),
            ));
        }
        split_table_name(&request.target_full_name)?;
    }
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

fn split_table_name(full_name: &str) -> Result<(String, String, String), AppError> {
    let parts = full_name.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(AppError::InvalidParameter(
            "table target_full_name must be catalog.schema.table".to_owned(),
        ));
    }
    Ok((
        parts[0].to_owned(),
        parts[1].to_owned(),
        parts[2].to_owned(),
    ))
}

fn custom_property<'a>(document: &'a Value, name: &str) -> Option<&'a Value> {
    document
        .get("customProperties")?
        .as_array()?
        .iter()
        .find(|property| property.get("property").and_then(Value::as_str) == Some(name))?
        .get("value")
}

fn has_quality_rules(document: &Value) -> bool {
    document
        .get("schema")
        .and_then(Value::as_array)
        .is_some_and(|schemas| {
            schemas.iter().any(|schema| {
                schema
                    .get("quality")
                    .and_then(Value::as_array)
                    .is_some_and(|rules| !rules.is_empty())
                    || schema
                        .get("properties")
                        .and_then(Value::as_array)
                        .is_some_and(|properties| {
                            properties.iter().any(|property| {
                                property
                                    .get("quality")
                                    .and_then(Value::as_array)
                                    .is_some_and(|rules| !rules.is_empty())
                            })
                        })
            })
        })
}

fn evaluate_contract(
    id: String,
    version: String,
    document: Value,
    document_hash: String,
    sync_state: String,
    latest_quality_status: String,
    context: &ConsumerContext,
    requested_duration_seconds: i64,
) -> Result<EvaluatedContract, AppError> {
    if document.get("status").and_then(Value::as_str) != Some("active") {
        return Err(AppError::Conflict(format!(
            "data contract {id} version {version} is not active"
        )));
    }
    if sync_state != "IN_SYNC" {
        return Err(AppError::Conflict(format!(
            "data contract {id} is {sync_state}; reconcile it before requesting access"
        )));
    }

    let residency_countries = custom_property(&document, "residencyCountries")
        .and_then(Value::as_array)
        .map(|countries| {
            countries
                .iter()
                .filter_map(Value::as_str)
                .map(|country| country.to_ascii_uppercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let outside_residency = context.processing_countries.iter().any(|country| {
        !residency_countries.is_empty()
            && !residency_countries.contains(&country.trim().to_ascii_uppercase())
    });
    let cross_border_allowed = custom_property(&document, "crossBorderTransferAllowed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if (context.cross_border_transfer || outside_residency) && !cross_border_allowed {
        return Err(AppError::Conflict(format!(
            "data contract {id} prohibits the requested cross-border processing"
        )));
    }
    if let Some(max_hours) =
        custom_property(&document, "maxAccessDurationHours").and_then(Value::as_i64)
        && requested_duration_seconds > max_hours * 3600
    {
        return Err(AppError::Conflict(format!(
            "requested duration exceeds the contract maximum of {max_hours} hours"
        )));
    }

    let mut evaluations = vec![
        ContractEvaluation {
            rule_key: "contract.active",
            name: "Active contract",
            enforcement: "BLOCK",
            status: "PASS",
            message: format!("Contract version {version} is active"),
            evidence: json!({ "version": &version, "documentHash": &document_hash }),
        },
        ContractEvaluation {
            rule_key: "schema.in_sync",
            name: "Unity Catalog schema",
            enforcement: "BLOCK",
            status: "PASS",
            message: "Contract and Unity Catalog schemas are in sync".to_owned(),
            evidence: json!({ "syncState": &sync_state }),
        },
        ContractEvaluation {
            rule_key: "residency.allowed",
            name: "Processing residency",
            enforcement: "BLOCK",
            status: "PASS",
            message: "Requested processing locations satisfy the contract".to_owned(),
            evidence: json!({
                "allowedCountries": residency_countries,
                "requestedCountries": &context.processing_countries,
                "crossBorderTransfer": context.cross_border_transfer,
            }),
        },
    ];

    if has_quality_rules(&document) {
        let (status, message) = match latest_quality_status.as_str() {
            "PASSED" => ("PASS", "Latest contract quality run passed"),
            "FAILED" | "ERROR" => ("WARN", "Latest contract quality run did not pass"),
            "RUNNING" => ("WARN", "Contract quality checks are still running"),
            _ => ("WARN", "Contract quality rules have not been run"),
        };
        evaluations.push(ContractEvaluation {
            rule_key: "quality.latest_run",
            name: "Latest data quality run",
            enforcement: "ADVISORY",
            status,
            message: message.to_owned(),
            evidence: json!({ "latestStatus": &latest_quality_status }),
        });
    }
    if context.ai_use {
        evaluations.push(ContractEvaluation {
            rule_key: "usage.ai",
            name: "AI or model use",
            enforcement: "ADVISORY",
            status: "WARN",
            message: "AI use was declared and requires reviewer attention".to_owned(),
            evidence: json!({ "aiUse": true }),
        });
    }
    let status = if evaluations
        .iter()
        .any(|evaluation| evaluation.status == "WARN")
    {
        "WARNING"
    } else {
        "PASSED"
    };
    let snapshot = json!({
        "status": document.get("status").cloned().unwrap_or(Value::Null),
        "domain": document.get("domain").cloned().unwrap_or(Value::Null),
        "dataProduct": document.get("dataProduct").cloned().unwrap_or(Value::Null),
        "description": document.get("description").cloned().unwrap_or(Value::Null),
        "team": document.get("team").cloned().unwrap_or(Value::Null),
        "slaProperties": document.get("slaProperties").cloned().unwrap_or_else(|| json!([])),
        "customProperties": document.get("customProperties").cloned().unwrap_or_else(|| json!([])),
        "syncState": &sync_state,
        "latestQualityStatus": &latest_quality_status,
    });
    Ok(EvaluatedContract {
        id,
        version,
        document_hash,
        snapshot,
        evaluations: serde_json::to_value(evaluations)?,
        status,
    })
}

fn contract_binding_error(
    active_version: &str,
    active_document_hash: &str,
    sync_state: &str,
    bound_version: &str,
    bound_document_hash: &str,
) -> Option<String> {
    if sync_state != "IN_SYNC" {
        return Some(format!(
            "data contract is {sync_state}; reconcile it before approving or granting access"
        ));
    }
    if active_version != bound_version || active_document_hash != bound_document_hash {
        return Some(format!(
            "data contract changed from version {bound_version} to {active_version}; submit a new request against the current terms"
        ));
    }
    None
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
        assert!(can_record_review("FULFILLED", "REJECTED"));
        assert!(!can_record_review("FULFILLED", "APPROVED"));
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
    fn completed_revocation_is_not_repeated() {
        for status in ["GRANTING", "GRANTED", "UNCHANGED"] {
            assert!(needs_revocation(status));
        }
        for status in ["PENDING", "FAILED", "REVOKED"] {
            assert!(!needs_revocation(status));
        }
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
        assert!(can_reject_auto_approved_request(
            "FULFILLED",
            "system:auto-approve",
            "analytics",
            "analytics",
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
            purpose_code: Some("ANALYTICS".to_owned()),
            consumer_context: Some(ConsumerContext {
                consumer_domain: "analytics".to_owned(),
                processing_countries: vec!["VN".to_owned()],
                cross_border_transfer: false,
                ai_use: false,
                intended_output: None,
            }),
            acknowledged_contract_terms: true,
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

    #[test]
    fn browse_is_requestable_for_catalog_schema_and_table() {
        for securable_type in ["catalog", "schema", "table"] {
            let mut request = valid_request();
            request.target_securable_type = securable_type.to_owned();
            request.requested_privilege = "BROWSE".to_owned();

            assert!(validate_create_request(&request).is_ok());
        }

        assert!(!privileges_for("metastore").contains(&"BROWSE"));
    }

    fn active_contract() -> Value {
        json!({
            "status": "active",
            "domain": "hdbank",
            "description": {
                "usage": "Governed analytics",
                "limitations": "Vietnam processing only"
            },
            "customProperties": [
                { "property": "residencyCountries", "value": ["VN"] },
                { "property": "crossBorderTransferAllowed", "value": false },
                { "property": "maxAccessDurationHours", "value": 24 }
            ],
            "schema": [{ "quality": [{ "metric": "rowCount" }] }]
        })
    }

    fn consumer_context() -> ConsumerContext {
        ConsumerContext {
            consumer_domain: "vietjet".to_owned(),
            processing_countries: vec!["VN".to_owned()],
            cross_border_transfer: false,
            ai_use: false,
            intended_output: Some("Group customer data product".to_owned()),
        }
    }

    #[test]
    fn contract_evaluation_blocks_processing_outside_residency() {
        let mut context = consumer_context();
        context.processing_countries = vec!["SG".to_owned()];

        let error = evaluate_contract(
            "contract-1".to_owned(),
            "1.0.0".to_owned(),
            active_contract(),
            "hash".to_owned(),
            "IN_SYNC".to_owned(),
            "PASSED".to_owned(),
            &context,
            3600,
        )
        .err()
        .expect("processing outside residency must fail");

        assert!(
            error
                .to_string()
                .contains("prohibits the requested cross-border")
        );
    }

    #[test]
    fn missing_quality_run_warns_and_prevents_auto_approval() {
        let evaluated = evaluate_contract(
            "contract-1".to_owned(),
            "1.0.0".to_owned(),
            active_contract(),
            "hash".to_owned(),
            "IN_SYNC".to_owned(),
            String::new(),
            &consumer_context(),
            3600,
        )
        .expect("advisory quality state should not block the request");

        assert_eq!(evaluated.status, "WARNING");
        assert!(
            evaluated
                .evaluations
                .as_array()
                .unwrap()
                .iter()
                .any(|item| {
                    item.get("rule_key").and_then(Value::as_str) == Some("quality.latest_run")
                        && item.get("status").and_then(Value::as_str) == Some("WARN")
                })
        );
    }

    #[test]
    fn fulfillment_rejects_a_changed_contract_binding() {
        assert!(
            contract_binding_error("2.0.0", "new", "IN_SYNC", "1.0.0", "old")
                .unwrap()
                .contains("changed from version 1.0.0 to 2.0.0")
        );
        assert!(contract_binding_error("1.0.0", "hash", "IN_SYNC", "1.0.0", "hash").is_none());
    }
}
