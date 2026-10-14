// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct ResolveRequestPolicyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub team: T1,
    pub target_securable_type: T2,
    pub target_full_name: T3,
    pub privilege: T4,
}
#[derive(Debug)]
pub struct CreateAccessRequestParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::StringSql,
    T9: crate::StringSql,
    T10: crate::StringSql,
    T11: crate::StringSql,
    T12: crate::StringSql,
    T13: crate::JsonSql,
    T14: crate::JsonSql,
    T15: crate::JsonSql,
    T16: crate::StringSql,
    T17: crate::StringSql,
    T18: crate::StringSql,
> {
    pub template_id: T1,
    pub template_version: i32,
    pub template_name: T2,
    pub approval_strategy: T3,
    pub requested_by: T4,
    pub requested_for: T5,
    pub routed_for_team: T6,
    pub target_securable_type: T7,
    pub target_full_name: T8,
    pub contract_id: T9,
    pub contract_version: T10,
    pub contract_document_hash: T11,
    pub purpose_code: T12,
    pub consumer_context: T13,
    pub contract_snapshot: T14,
    pub contract_evaluations: T15,
    pub contract_evaluation_status: T16,
    pub reason: T17,
    pub auto_approve: bool,
    pub requested_duration_seconds: i64,
    pub idempotency_key: T18,
}
#[derive(Debug)]
pub struct GetActiveContractForAccessRequestParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub table_name: T3,
}
#[derive(Debug)]
pub struct FailAccessRequestContractValidationParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub message: T1,
    pub request_id: T2,
}
#[derive(Debug)]
pub struct CopyAccessRequestApprovalTeamsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub template_id: T2,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct CopyAccessRequestGrantsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub template_id: T2,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct CreateAccessRequestApprovalTeamParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub team: T2,
}
#[derive(Debug)]
pub struct CreateAccessRequestGrantParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub request_id: T1,
    pub securable_type: T2,
    pub full_name: T3,
    pub privilege: T4,
}
#[derive(Debug)]
pub struct CopyAccessRequestGuardrailsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub template_id: T2,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct CreateAccessRequestEventParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub request_id: T1,
    pub event_type: T2,
    pub actor: T3,
    pub comment: T4,
}
#[derive(Debug)]
pub struct ListAccessRequestsParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub status: T1,
    pub requested_by: T2,
    pub requested_for: T3,
    pub approval_team: T4,
}
#[derive(Debug)]
pub struct SetAccessRequestGrantStatusParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
> {
    pub status: T1,
    pub failure_message: T2,
    pub request_id: T3,
    pub securable_type: T4,
    pub full_name: T5,
    pub privilege: T6,
}
#[derive(Debug)]
pub struct FailPendingAccessRequestGrantsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub failure_message: T1,
    pub request_id: T2,
}
#[derive(Debug)]
pub struct HasOtherActiveManagedGrantParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub request_id: T1,
    pub requested_for: T2,
    pub securable_type: T3,
    pub full_name: T4,
    pub privilege: T5,
}
#[derive(Debug)]
pub struct LockOwnedAccessRequestParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub requested_by: T2,
}
#[derive(Debug)]
pub struct UpdateAccessRequestParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub reason: T1,
    pub requested_duration_seconds: i64,
    pub request_id: T2,
}
#[derive(Debug)]
pub struct CancelAccessRequestParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub requested_by: T1,
    pub request_id: T2,
}
#[derive(Debug)]
pub struct IsAccessRequestApprovalTeamParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub request_id: T1,
    pub approval_team: T2,
}
#[derive(Debug)]
pub struct CreateAccessRequestReviewParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub request_id: T1,
    pub approval_team: T2,
    pub reviewer: T3,
    pub decision: T4,
    pub comment: T5,
}
#[derive(Debug)]
pub struct ApproveAccessRequestParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub reviewer: T1,
    pub comment: T2,
    pub request_id: T3,
}
#[derive(Debug)]
pub struct RejectAccessRequestParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub reviewer: T1,
    pub comment: T2,
    pub request_id: T3,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ResolveRequestPolicy {
    pub template_id: String,
    pub version: i32,
    pub name: String,
    pub approval_strategy: String,
    pub auto_approve: bool,
}
pub struct ResolveRequestPolicyBorrowed<'a> {
    pub template_id: &'a str,
    pub version: i32,
    pub name: &'a str,
    pub approval_strategy: &'a str,
    pub auto_approve: bool,
}
impl<'a> From<ResolveRequestPolicyBorrowed<'a>> for ResolveRequestPolicy {
    fn from(
        ResolveRequestPolicyBorrowed {
            template_id,
            version,
            name,
            approval_strategy,
            auto_approve,
        }: ResolveRequestPolicyBorrowed<'a>,
    ) -> Self {
        Self {
            template_id: template_id.into(),
            version,
            name: name.into(),
            approval_strategy: approval_strategy.into(),
            auto_approve,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetActiveContractForAccessRequest {
    pub id: String,
    pub version: String,
    pub document: serde_json::Value,
    pub document_hash: String,
    pub sync_state: String,
    pub latest_quality_status: String,
}
pub struct GetActiveContractForAccessRequestBorrowed<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub document: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub document_hash: &'a str,
    pub sync_state: &'a str,
    pub latest_quality_status: &'a str,
}
impl<'a> From<GetActiveContractForAccessRequestBorrowed<'a>> for GetActiveContractForAccessRequest {
    fn from(
        GetActiveContractForAccessRequestBorrowed {
            id,
            version,
            document,
            document_hash,
            sync_state,
            latest_quality_status,
        }: GetActiveContractForAccessRequestBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            document: serde_json::from_str(document.0.get()).unwrap(),
            document_hash: document_hash.into(),
            sync_state: sync_state.into(),
            latest_quality_status: latest_quality_status.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetAccessRequestContractState {
    pub active_version: String,
    pub active_document_hash: String,
    pub sync_state: String,
    pub bound_version: String,
    pub bound_document_hash: String,
}
pub struct GetAccessRequestContractStateBorrowed<'a> {
    pub active_version: &'a str,
    pub active_document_hash: &'a str,
    pub sync_state: &'a str,
    pub bound_version: &'a str,
    pub bound_document_hash: &'a str,
}
impl<'a> From<GetAccessRequestContractStateBorrowed<'a>> for GetAccessRequestContractState {
    fn from(
        GetAccessRequestContractStateBorrowed {
            active_version,
            active_document_hash,
            sync_state,
            bound_version,
            bound_document_hash,
        }: GetAccessRequestContractStateBorrowed<'a>,
    ) -> Self {
        Self {
            active_version: active_version.into(),
            active_document_hash: active_document_hash.into(),
            sync_state: sync_state.into(),
            bound_version: bound_version.into(),
            bound_document_hash: bound_document_hash.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequests {
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
    pub decided_by: String,
    pub decided_at: String,
    pub decision_comment: String,
}
pub struct ListAccessRequestsBorrowed<'a> {
    pub id: &'a str,
    pub template_id: &'a str,
    pub template_version: i32,
    pub template_name: &'a str,
    pub requested_by: &'a str,
    pub requested_for: &'a str,
    pub routed_for_team: &'a str,
    pub target_securable_type: &'a str,
    pub target_full_name: &'a str,
    pub reason: &'a str,
    pub status: &'a str,
    pub requested_duration_seconds: i64,
    pub submitted_at: &'a str,
    pub decided_by: &'a str,
    pub decided_at: &'a str,
    pub decision_comment: &'a str,
}
impl<'a> From<ListAccessRequestsBorrowed<'a>> for ListAccessRequests {
    fn from(
        ListAccessRequestsBorrowed {
            id,
            template_id,
            template_version,
            template_name,
            requested_by,
            requested_for,
            routed_for_team,
            target_securable_type,
            target_full_name,
            reason,
            status,
            requested_duration_seconds,
            submitted_at,
            decided_by,
            decided_at,
            decision_comment,
        }: ListAccessRequestsBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            template_id: template_id.into(),
            template_version,
            template_name: template_name.into(),
            requested_by: requested_by.into(),
            requested_for: requested_for.into(),
            routed_for_team: routed_for_team.into(),
            target_securable_type: target_securable_type.into(),
            target_full_name: target_full_name.into(),
            reason: reason.into(),
            status: status.into(),
            requested_duration_seconds,
            submitted_at: submitted_at.into(),
            decided_by: decided_by.into(),
            decided_at: decided_at.into(),
            decision_comment: decision_comment.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListExpiredFulfilledAccessRequests {
    pub id: String,
    pub requested_for: String,
}
pub struct ListExpiredFulfilledAccessRequestsBorrowed<'a> {
    pub id: &'a str,
    pub requested_for: &'a str,
}
impl<'a> From<ListExpiredFulfilledAccessRequestsBorrowed<'a>>
    for ListExpiredFulfilledAccessRequests
{
    fn from(
        ListExpiredFulfilledAccessRequestsBorrowed {
            id,
            requested_for,
        }: ListExpiredFulfilledAccessRequestsBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            requested_for: requested_for.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestsPendingRevocation {
    pub id: String,
    pub requested_for: String,
}
pub struct ListAccessRequestsPendingRevocationBorrowed<'a> {
    pub id: &'a str,
    pub requested_for: &'a str,
}
impl<'a> From<ListAccessRequestsPendingRevocationBorrowed<'a>>
    for ListAccessRequestsPendingRevocation
{
    fn from(
        ListAccessRequestsPendingRevocationBorrowed {
            id,
            requested_for,
        }: ListAccessRequestsPendingRevocationBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            requested_for: requested_for.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetAccessRequest {
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
    pub contract_id: String,
    pub contract_version: String,
    pub contract_document_hash: String,
    pub purpose_code: String,
    pub consumer_context: serde_json::Value,
    pub contract_snapshot: serde_json::Value,
    pub contract_evaluations: serde_json::Value,
    pub contract_evaluation_status: String,
    pub contract_evaluated_at: String,
    pub reason: String,
    pub status: String,
    pub requested_duration_seconds: i64,
    pub submitted_at: String,
    pub decided_by: String,
    pub decided_at: String,
    pub decision_comment: String,
    pub fulfilled_at: String,
    pub access_expires_at: String,
}
pub struct GetAccessRequestBorrowed<'a> {
    pub id: &'a str,
    pub template_id: &'a str,
    pub template_version: i32,
    pub template_name: &'a str,
    pub approval_strategy: &'a str,
    pub requested_by: &'a str,
    pub requested_for: &'a str,
    pub routed_for_team: &'a str,
    pub target_securable_type: &'a str,
    pub target_full_name: &'a str,
    pub contract_id: &'a str,
    pub contract_version: &'a str,
    pub contract_document_hash: &'a str,
    pub purpose_code: &'a str,
    pub consumer_context: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub contract_snapshot: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub contract_evaluations: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub contract_evaluation_status: &'a str,
    pub contract_evaluated_at: &'a str,
    pub reason: &'a str,
    pub status: &'a str,
    pub requested_duration_seconds: i64,
    pub submitted_at: &'a str,
    pub decided_by: &'a str,
    pub decided_at: &'a str,
    pub decision_comment: &'a str,
    pub fulfilled_at: &'a str,
    pub access_expires_at: &'a str,
}
impl<'a> From<GetAccessRequestBorrowed<'a>> for GetAccessRequest {
    fn from(
        GetAccessRequestBorrowed {
            id,
            template_id,
            template_version,
            template_name,
            approval_strategy,
            requested_by,
            requested_for,
            routed_for_team,
            target_securable_type,
            target_full_name,
            contract_id,
            contract_version,
            contract_document_hash,
            purpose_code,
            consumer_context,
            contract_snapshot,
            contract_evaluations,
            contract_evaluation_status,
            contract_evaluated_at,
            reason,
            status,
            requested_duration_seconds,
            submitted_at,
            decided_by,
            decided_at,
            decision_comment,
            fulfilled_at,
            access_expires_at,
        }: GetAccessRequestBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            template_id: template_id.into(),
            template_version,
            template_name: template_name.into(),
            approval_strategy: approval_strategy.into(),
            requested_by: requested_by.into(),
            requested_for: requested_for.into(),
            routed_for_team: routed_for_team.into(),
            target_securable_type: target_securable_type.into(),
            target_full_name: target_full_name.into(),
            contract_id: contract_id.into(),
            contract_version: contract_version.into(),
            contract_document_hash: contract_document_hash.into(),
            purpose_code: purpose_code.into(),
            consumer_context: serde_json::from_str(consumer_context.0.get()).unwrap(),
            contract_snapshot: serde_json::from_str(contract_snapshot.0.get()).unwrap(),
            contract_evaluations: serde_json::from_str(contract_evaluations.0.get()).unwrap(),
            contract_evaluation_status: contract_evaluation_status.into(),
            contract_evaluated_at: contract_evaluated_at.into(),
            reason: reason.into(),
            status: status.into(),
            requested_duration_seconds,
            submitted_at: submitted_at.into(),
            decided_by: decided_by.into(),
            decided_at: decided_at.into(),
            decision_comment: decision_comment.into(),
            fulfilled_at: fulfilled_at.into(),
            access_expires_at: access_expires_at.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct LockAccessRequest {
    pub status: String,
    pub approval_strategy: String,
    pub requested_for: String,
    pub routed_for_team: String,
    pub decided_by: String,
}
pub struct LockAccessRequestBorrowed<'a> {
    pub status: &'a str,
    pub approval_strategy: &'a str,
    pub requested_for: &'a str,
    pub routed_for_team: &'a str,
    pub decided_by: &'a str,
}
impl<'a> From<LockAccessRequestBorrowed<'a>> for LockAccessRequest {
    fn from(
        LockAccessRequestBorrowed {
            status,
            approval_strategy,
            requested_for,
            routed_for_team,
            decided_by,
        }: LockAccessRequestBorrowed<'a>,
    ) -> Self {
        Self {
            status: status.into(),
            approval_strategy: approval_strategy.into(),
            requested_for: requested_for.into(),
            routed_for_team: routed_for_team.into(),
            decided_by: decided_by.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct LockOwnedAccessRequest {
    pub status: String,
    pub contract_document: serde_json::Value,
}
pub struct LockOwnedAccessRequestBorrowed<'a> {
    pub status: &'a str,
    pub contract_document: postgres_types::Json<&'a serde_json::value::RawValue>,
}
impl<'a> From<LockOwnedAccessRequestBorrowed<'a>> for LockOwnedAccessRequest {
    fn from(
        LockOwnedAccessRequestBorrowed {
            status,
            contract_document,
        }: LockOwnedAccessRequestBorrowed<'a>,
    ) -> Self {
        Self {
            status: status.into(),
            contract_document: serde_json::from_str(contract_document.0.get()).unwrap(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestApprovalTeams {
    pub team: String,
    pub required_approvals: i32,
    pub approved_count: i32,
    pub rejected_count: i32,
}
pub struct ListAccessRequestApprovalTeamsBorrowed<'a> {
    pub team: &'a str,
    pub required_approvals: i32,
    pub approved_count: i32,
    pub rejected_count: i32,
}
impl<'a> From<ListAccessRequestApprovalTeamsBorrowed<'a>> for ListAccessRequestApprovalTeams {
    fn from(
        ListAccessRequestApprovalTeamsBorrowed {
            team,
            required_approvals,
            approved_count,
            rejected_count,
        }: ListAccessRequestApprovalTeamsBorrowed<'a>,
    ) -> Self {
        Self {
            team: team.into(),
            required_approvals,
            approved_count,
            rejected_count,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestGrants {
    pub securable_type: String,
    pub full_name: String,
    pub privilege: String,
    pub fulfillment_status: String,
}
pub struct ListAccessRequestGrantsBorrowed<'a> {
    pub securable_type: &'a str,
    pub full_name: &'a str,
    pub privilege: &'a str,
    pub fulfillment_status: &'a str,
}
impl<'a> From<ListAccessRequestGrantsBorrowed<'a>> for ListAccessRequestGrants {
    fn from(
        ListAccessRequestGrantsBorrowed {
            securable_type,
            full_name,
            privilege,
            fulfillment_status,
        }: ListAccessRequestGrantsBorrowed<'a>,
    ) -> Self {
        Self {
            securable_type: securable_type.into(),
            full_name: full_name.into(),
            privilege: privilege.into(),
            fulfillment_status: fulfillment_status.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestGuardrails {
    pub guardrail_key: String,
    pub name: String,
    pub description: String,
    pub enforcement: String,
    pub configuration: serde_json::Value,
    pub evaluation_status: String,
}
pub struct ListAccessRequestGuardrailsBorrowed<'a> {
    pub guardrail_key: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub enforcement: &'a str,
    pub configuration: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub evaluation_status: &'a str,
}
impl<'a> From<ListAccessRequestGuardrailsBorrowed<'a>> for ListAccessRequestGuardrails {
    fn from(
        ListAccessRequestGuardrailsBorrowed {
            guardrail_key,
            name,
            description,
            enforcement,
            configuration,
            evaluation_status,
        }: ListAccessRequestGuardrailsBorrowed<'a>,
    ) -> Self {
        Self {
            guardrail_key: guardrail_key.into(),
            name: name.into(),
            description: description.into(),
            enforcement: enforcement.into(),
            configuration: serde_json::from_str(configuration.0.get()).unwrap(),
            evaluation_status: evaluation_status.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestReviews {
    pub approval_team: String,
    pub reviewer: String,
    pub decision: String,
    pub comment: String,
    pub created_at: String,
}
pub struct ListAccessRequestReviewsBorrowed<'a> {
    pub approval_team: &'a str,
    pub reviewer: &'a str,
    pub decision: &'a str,
    pub comment: &'a str,
    pub created_at: &'a str,
}
impl<'a> From<ListAccessRequestReviewsBorrowed<'a>> for ListAccessRequestReviews {
    fn from(
        ListAccessRequestReviewsBorrowed {
            approval_team,
            reviewer,
            decision,
            comment,
            created_at,
        }: ListAccessRequestReviewsBorrowed<'a>,
    ) -> Self {
        Self {
            approval_team: approval_team.into(),
            reviewer: reviewer.into(),
            decision: decision.into(),
            comment: comment.into(),
            created_at: created_at.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListAccessRequestEvents {
    pub event_type: String,
    pub actor: String,
    pub comment: String,
    pub created_at: String,
}
pub struct ListAccessRequestEventsBorrowed<'a> {
    pub event_type: &'a str,
    pub actor: &'a str,
    pub comment: &'a str,
    pub created_at: &'a str,
}
impl<'a> From<ListAccessRequestEventsBorrowed<'a>> for ListAccessRequestEvents {
    fn from(
        ListAccessRequestEventsBorrowed {
            event_type,
            actor,
            comment,
            created_at,
        }: ListAccessRequestEventsBorrowed<'a>,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            actor: actor.into(),
            comment: comment.into(),
            created_at: created_at.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ResolveRequestPolicyQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ResolveRequestPolicyBorrowed, tokio_postgres::Error>,
    mapper: fn(ResolveRequestPolicyBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ResolveRequestPolicyQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ResolveRequestPolicyBorrowed) -> R,
    ) -> ResolveRequestPolicyQuery<'c, 'a, 's, C, R, N> {
        ResolveRequestPolicyQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct StringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&str, tokio_postgres::Error>,
    mapper: fn(&str) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&str) -> R) -> StringQuery<'c, 'a, 's, C, R, N> {
        StringQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct GetActiveContractForAccessRequestQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<GetActiveContractForAccessRequestBorrowed, tokio_postgres::Error>,
    mapper: fn(GetActiveContractForAccessRequestBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize>
    GetActiveContractForAccessRequestQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetActiveContractForAccessRequestBorrowed) -> R,
    ) -> GetActiveContractForAccessRequestQuery<'c, 'a, 's, C, R, N> {
        GetActiveContractForAccessRequestQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct GetAccessRequestContractStateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<GetAccessRequestContractStateBorrowed, tokio_postgres::Error>,
    mapper: fn(GetAccessRequestContractStateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetAccessRequestContractStateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetAccessRequestContractStateBorrowed) -> R,
    ) -> GetAccessRequestContractStateQuery<'c, 'a, 's, C, R, N> {
        GetAccessRequestContractStateQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct I64Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<i64, tokio_postgres::Error>,
    mapper: fn(i64) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> I64Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(i64) -> R) -> I64Query<'c, 'a, 's, C, R, N> {
        I64Query {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListAccessRequestsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestsBorrowed) -> R,
    ) -> ListAccessRequestsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListExpiredFulfilledAccessRequestsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize>
{
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<ListExpiredFulfilledAccessRequestsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListExpiredFulfilledAccessRequestsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize>
    ListExpiredFulfilledAccessRequestsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListExpiredFulfilledAccessRequestsBorrowed) -> R,
    ) -> ListExpiredFulfilledAccessRequestsQuery<'c, 'a, 's, C, R, N> {
        ListExpiredFulfilledAccessRequestsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestsPendingRevocationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize>
{
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    )
        -> Result<ListAccessRequestsPendingRevocationBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestsPendingRevocationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize>
    ListAccessRequestsPendingRevocationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestsPendingRevocationBorrowed) -> R,
    ) -> ListAccessRequestsPendingRevocationQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestsPendingRevocationQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct BoolQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<bool, tokio_postgres::Error>,
    mapper: fn(bool) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> BoolQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(bool) -> R) -> BoolQuery<'c, 'a, 's, C, R, N> {
        BoolQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct GetAccessRequestQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetAccessRequestBorrowed, tokio_postgres::Error>,
    mapper: fn(GetAccessRequestBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetAccessRequestQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetAccessRequestBorrowed) -> R,
    ) -> GetAccessRequestQuery<'c, 'a, 's, C, R, N> {
        GetAccessRequestQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct LockAccessRequestQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<LockAccessRequestBorrowed, tokio_postgres::Error>,
    mapper: fn(LockAccessRequestBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> LockAccessRequestQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(LockAccessRequestBorrowed) -> R,
    ) -> LockAccessRequestQuery<'c, 'a, 's, C, R, N> {
        LockAccessRequestQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct LockOwnedAccessRequestQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<LockOwnedAccessRequestBorrowed, tokio_postgres::Error>,
    mapper: fn(LockOwnedAccessRequestBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> LockOwnedAccessRequestQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(LockOwnedAccessRequestBorrowed) -> R,
    ) -> LockOwnedAccessRequestQuery<'c, 'a, 's, C, R, N> {
        LockOwnedAccessRequestQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestApprovalTeamsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<ListAccessRequestApprovalTeamsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestApprovalTeamsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestApprovalTeamsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestApprovalTeamsBorrowed) -> R,
    ) -> ListAccessRequestApprovalTeamsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestApprovalTeamsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestGrantsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListAccessRequestGrantsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestGrantsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestGrantsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestGrantsBorrowed) -> R,
    ) -> ListAccessRequestGrantsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestGrantsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestGuardrailsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<ListAccessRequestGuardrailsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestGuardrailsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestGuardrailsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestGuardrailsBorrowed) -> R,
    ) -> ListAccessRequestGuardrailsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestGuardrailsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestReviewsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListAccessRequestReviewsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestReviewsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestReviewsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestReviewsBorrowed) -> R,
    ) -> ListAccessRequestReviewsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestReviewsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListAccessRequestEventsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListAccessRequestEventsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListAccessRequestEventsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListAccessRequestEventsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListAccessRequestEventsBorrowed) -> R,
    ) -> ListAccessRequestEventsQuery<'c, 'a, 's, C, R, N> {
        ListAccessRequestEventsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ResolveRequestPolicyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn resolve_request_policy() -> ResolveRequestPolicyStmt {
    ResolveRequestPolicyStmt(
        "SELECT versions.template_id, versions.version, versions.name, versions.approval_strategy, versions.auto_approve FROM policy_template_versions versions JOIN policy_template_affected_teams affected ON affected.template_id = versions.template_id AND affected.template_version = versions.version JOIN policy_template_grants policy_grant ON policy_grant.template_id = versions.template_id AND policy_grant.template_version = versions.version WHERE versions.status = 'PUBLISHED' AND affected.team = $1::text AND policy_grant.securable_type = $2::text AND policy_grant.target_full_name = $3::text AND policy_grant.privilege = $4::text",
        None,
    )
}
impl ResolveRequestPolicyStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        team: &'a T1,
        target_securable_type: &'a T2,
        target_full_name: &'a T3,
        privilege: &'a T4,
    ) -> ResolveRequestPolicyQuery<'c, 'a, 's, C, ResolveRequestPolicy, 4> {
        ResolveRequestPolicyQuery {
            client,
            params: [team, target_securable_type, target_full_name, privilege],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ResolveRequestPolicyBorrowed, tokio_postgres::Error> {
                Ok(ResolveRequestPolicyBorrowed {
                    template_id: row.try_get(0)?,
                    version: row.try_get(1)?,
                    name: row.try_get(2)?,
                    approval_strategy: row.try_get(3)?,
                    auto_approve: row.try_get(4)?,
                })
            },
            mapper: |it| ResolveRequestPolicy::from(it),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ResolveRequestPolicyParams<T1, T2, T3, T4>,
        ResolveRequestPolicyQuery<'c, 'a, 's, C, ResolveRequestPolicy, 4>,
        C,
    > for ResolveRequestPolicyStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ResolveRequestPolicyParams<T1, T2, T3, T4>,
    ) -> ResolveRequestPolicyQuery<'c, 'a, 's, C, ResolveRequestPolicy, 4> {
        self.bind(
            client,
            &params.team,
            &params.target_securable_type,
            &params.target_full_name,
            &params.privilege,
        )
    }
}
pub struct CreateAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_access_request() -> CreateAccessRequestStmt {
    CreateAccessRequestStmt(
        "INSERT INTO access_requests ( template_id, template_version, template_name, approval_strategy, requested_by, requested_for, routed_for_team, target_securable_type, target_full_name, contract_id, contract_version, contract_document_hash, purpose_code, consumer_context, contract_snapshot, contract_evaluations, contract_evaluation_status, contract_evaluated_at, reason, status, requested_duration_seconds, idempotency_key, decided_by, decided_at, decision_comment, access_expires_at ) VALUES ( NULLIF($1::text, ''), CASE WHEN $1::text = '' THEN NULL ELSE $2::integer END, $3::text, $4::text, $5::text, $6::text, $7::text, $8::text, $9::text, NULLIF($10::text, ''), NULLIF($11::text, ''), NULLIF($12::text, ''), NULLIF($13::text, ''), $14, $15, $16, $17::text, CASE WHEN $10::text = '' THEN NULL ELSE NOW() END, $18::text, CASE WHEN $19::boolean THEN 'APPROVED' ELSE 'AWAITING_APPROVAL' END, $20::bigint, NULLIF($21::text, ''), CASE WHEN $19::boolean THEN 'system:auto-approve' END, CASE WHEN $19::boolean THEN NOW() END, CASE WHEN $19::boolean THEN 'Automatically approved by policy template' END, CASE WHEN $19::boolean THEN NOW() + ($20::bigint * INTERVAL '1 second') END ) RETURNING id",
        None,
    )
}
impl CreateAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
        T6: crate::StringSql,
        T7: crate::StringSql,
        T8: crate::StringSql,
        T9: crate::StringSql,
        T10: crate::StringSql,
        T11: crate::StringSql,
        T12: crate::StringSql,
        T13: crate::JsonSql,
        T14: crate::JsonSql,
        T15: crate::JsonSql,
        T16: crate::StringSql,
        T17: crate::StringSql,
        T18: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        template_id: &'a T1,
        template_version: &'a i32,
        template_name: &'a T2,
        approval_strategy: &'a T3,
        requested_by: &'a T4,
        requested_for: &'a T5,
        routed_for_team: &'a T6,
        target_securable_type: &'a T7,
        target_full_name: &'a T8,
        contract_id: &'a T9,
        contract_version: &'a T10,
        contract_document_hash: &'a T11,
        purpose_code: &'a T12,
        consumer_context: &'a T13,
        contract_snapshot: &'a T14,
        contract_evaluations: &'a T15,
        contract_evaluation_status: &'a T16,
        reason: &'a T17,
        auto_approve: &'a bool,
        requested_duration_seconds: &'a i64,
        idempotency_key: &'a T18,
    ) -> StringQuery<'c, 'a, 's, C, String, 21> {
        StringQuery {
            client,
            params: [
                template_id,
                template_version,
                template_name,
                approval_strategy,
                requested_by,
                requested_for,
                routed_for_team,
                target_securable_type,
                target_full_name,
                contract_id,
                contract_version,
                contract_document_hash,
                purpose_code,
                consumer_context,
                contract_snapshot,
                contract_evaluations,
                contract_evaluation_status,
                reason,
                auto_approve,
                requested_duration_seconds,
                idempotency_key,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::StringSql,
    T9: crate::StringSql,
    T10: crate::StringSql,
    T11: crate::StringSql,
    T12: crate::StringSql,
    T13: crate::JsonSql,
    T14: crate::JsonSql,
    T15: crate::JsonSql,
    T16: crate::StringSql,
    T17: crate::StringSql,
    T18: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateAccessRequestParams<
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
            T13,
            T14,
            T15,
            T16,
            T17,
            T18,
        >,
        StringQuery<'c, 'a, 's, C, String, 21>,
        C,
    > for CreateAccessRequestStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateAccessRequestParams<
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
            T13,
            T14,
            T15,
            T16,
            T17,
            T18,
        >,
    ) -> StringQuery<'c, 'a, 's, C, String, 21> {
        self.bind(
            client,
            &params.template_id,
            &params.template_version,
            &params.template_name,
            &params.approval_strategy,
            &params.requested_by,
            &params.requested_for,
            &params.routed_for_team,
            &params.target_securable_type,
            &params.target_full_name,
            &params.contract_id,
            &params.contract_version,
            &params.contract_document_hash,
            &params.purpose_code,
            &params.consumer_context,
            &params.contract_snapshot,
            &params.contract_evaluations,
            &params.contract_evaluation_status,
            &params.reason,
            &params.auto_approve,
            &params.requested_duration_seconds,
            &params.idempotency_key,
        )
    }
}
pub struct GetActiveContractForAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_active_contract_for_access_request() -> GetActiveContractForAccessRequestStmt {
    GetActiveContractForAccessRequestStmt(
        "SELECT contracts.id, versions.version, versions.document, versions.document_hash, COALESCE(observations.state, 'PENDING_DEPLOYMENT') AS sync_state, COALESCE(( SELECT runs.status FROM data_quality_runs runs WHERE runs.contract_id = contracts.id AND runs.contract_version = versions.version ORDER BY runs.started_at DESC LIMIT 1 ), '') AS latest_quality_status FROM data_contracts contracts JOIN data_contract_versions versions ON versions.contract_id = contracts.id AND versions.version = contracts.active_version LEFT JOIN data_contract_observations observations ON observations.contract_id = contracts.id AND observations.contract_version = versions.version WHERE contracts.catalog_name = $1::text AND contracts.schema_name = $2::text AND contracts.table_name = $3::text",
        None,
    )
}
impl GetActiveContractForAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        catalog_name: &'a T1,
        schema_name: &'a T2,
        table_name: &'a T3,
    ) -> GetActiveContractForAccessRequestQuery<'c, 'a, 's, C, GetActiveContractForAccessRequest, 3>
    {
        GetActiveContractForAccessRequestQuery {
            client,
            params: [catalog_name, schema_name, table_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                GetActiveContractForAccessRequestBorrowed,
                tokio_postgres::Error,
            > {
                Ok(GetActiveContractForAccessRequestBorrowed {
                    id: row.try_get(0)?,
                    version: row.try_get(1)?,
                    document: row.try_get(2)?,
                    document_hash: row.try_get(3)?,
                    sync_state: row.try_get(4)?,
                    latest_quality_status: row.try_get(5)?,
                })
            },
            mapper: |it| GetActiveContractForAccessRequest::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetActiveContractForAccessRequestParams<T1, T2, T3>,
        GetActiveContractForAccessRequestQuery<'c, 'a, 's, C, GetActiveContractForAccessRequest, 3>,
        C,
    > for GetActiveContractForAccessRequestStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetActiveContractForAccessRequestParams<T1, T2, T3>,
    ) -> GetActiveContractForAccessRequestQuery<'c, 'a, 's, C, GetActiveContractForAccessRequest, 3>
    {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.table_name,
        )
    }
}
pub struct GetAccessRequestContractStateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_access_request_contract_state() -> GetAccessRequestContractStateStmt {
    GetAccessRequestContractStateStmt(
        "SELECT COALESCE(contracts.active_version, '') AS active_version, COALESCE(active_version.document_hash, '') AS active_document_hash, COALESCE(observations.state, 'PENDING_DEPLOYMENT') AS sync_state, COALESCE(requests.contract_version, '') AS bound_version, COALESCE(requests.contract_document_hash, '') AS bound_document_hash FROM access_requests requests JOIN data_contracts contracts ON contracts.id = requests.contract_id LEFT JOIN data_contract_versions active_version ON active_version.contract_id = contracts.id AND active_version.version = contracts.active_version LEFT JOIN data_contract_observations observations ON observations.contract_id = contracts.id AND observations.contract_version = contracts.active_version WHERE requests.id = $1::text AND requests.contract_id IS NOT NULL",
        None,
    )
}
impl GetAccessRequestContractStateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> GetAccessRequestContractStateQuery<'c, 'a, 's, C, GetAccessRequestContractState, 1> {
        GetAccessRequestContractStateQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetAccessRequestContractStateBorrowed, tokio_postgres::Error> {
                Ok(GetAccessRequestContractStateBorrowed {
                    active_version: row.try_get(0)?,
                    active_document_hash: row.try_get(1)?,
                    sync_state: row.try_get(2)?,
                    bound_version: row.try_get(3)?,
                    bound_document_hash: row.try_get(4)?,
                })
            },
            mapper: |it| GetAccessRequestContractState::from(it),
        }
    }
}
pub struct FailAccessRequestContractValidationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fail_access_request_contract_validation() -> FailAccessRequestContractValidationStmt {
    FailAccessRequestContractValidationStmt(
        "UPDATE access_requests SET status = 'FAILED', decided_by = 'system:contract-evaluation', decided_at = NOW(), decision_comment = $1::text, access_expires_at = NOW() WHERE id = $2::text AND status IN ('APPROVED', 'FULFILLING')",
        None,
    )
}
impl FailAccessRequestContractValidationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        message: &'a T1,
        request_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[message, request_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        FailAccessRequestContractValidationParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for FailAccessRequestContractValidationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a FailAccessRequestContractValidationParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.message, &params.request_id))
    }
}
pub struct CopyAccessRequestApprovalTeamsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn copy_access_request_approval_teams() -> CopyAccessRequestApprovalTeamsStmt {
    CopyAccessRequestApprovalTeamsStmt(
        "INSERT INTO access_request_approval_teams (request_id, team, required_approvals) SELECT $1::text, team, required_approvals FROM policy_template_approval_teams WHERE template_id = $2::text AND template_version = $3",
        None,
    )
}
impl CopyAccessRequestApprovalTeamsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        template_id: &'a T2,
        template_version: &'a i32,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[request_id, template_id, template_version])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CopyAccessRequestApprovalTeamsParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CopyAccessRequestApprovalTeamsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CopyAccessRequestApprovalTeamsParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.template_id,
            &params.template_version,
        ))
    }
}
pub struct CopyAccessRequestGrantsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn copy_access_request_grants() -> CopyAccessRequestGrantsStmt {
    CopyAccessRequestGrantsStmt(
        "INSERT INTO access_request_grants (request_id, securable_type, full_name, privilege) SELECT $1::text, securable_type, target_full_name, privilege FROM policy_template_grants WHERE template_id = $2::text AND template_version = $3",
        None,
    )
}
impl CopyAccessRequestGrantsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        template_id: &'a T2,
        template_version: &'a i32,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[request_id, template_id, template_version])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CopyAccessRequestGrantsParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CopyAccessRequestGrantsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CopyAccessRequestGrantsParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.template_id,
            &params.template_version,
        ))
    }
}
pub struct CreateAccessRequestApprovalTeamStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_access_request_approval_team() -> CreateAccessRequestApprovalTeamStmt {
    CreateAccessRequestApprovalTeamStmt(
        "INSERT INTO access_request_approval_teams (request_id, team, required_approvals) VALUES ($1::text, $2::text, 1)",
        None,
    )
}
impl CreateAccessRequestApprovalTeamStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        team: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[request_id, team]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateAccessRequestApprovalTeamParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateAccessRequestApprovalTeamStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateAccessRequestApprovalTeamParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.request_id, &params.team))
    }
}
pub struct CreateAccessRequestGrantStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_access_request_grant() -> CreateAccessRequestGrantStmt {
    CreateAccessRequestGrantStmt(
        "INSERT INTO access_request_grants (request_id, securable_type, full_name, privilege) VALUES ( $1::text, $2::text, $3::text, $4::text )",
        None,
    )
}
impl CreateAccessRequestGrantStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        securable_type: &'a T2,
        full_name: &'a T3,
        privilege: &'a T4,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[request_id, securable_type, full_name, privilege])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateAccessRequestGrantParams<T1, T2, T3, T4>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateAccessRequestGrantStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateAccessRequestGrantParams<T1, T2, T3, T4>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.securable_type,
            &params.full_name,
            &params.privilege,
        ))
    }
}
pub struct CopyAccessRequestGuardrailsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn copy_access_request_guardrails() -> CopyAccessRequestGuardrailsStmt {
    CopyAccessRequestGuardrailsStmt(
        "INSERT INTO access_request_guardrails ( request_id, source_template_guardrail_id, guardrail_key, name, description, enforcement, configuration ) SELECT $1::text, id, guardrail_key, name, description, enforcement, configuration FROM policy_template_guardrails WHERE template_id = $2::text AND template_version = $3",
        None,
    )
}
impl CopyAccessRequestGuardrailsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        template_id: &'a T2,
        template_version: &'a i32,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[request_id, template_id, template_version])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CopyAccessRequestGuardrailsParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CopyAccessRequestGuardrailsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CopyAccessRequestGuardrailsParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.template_id,
            &params.template_version,
        ))
    }
}
pub struct CountAccessRequestGrantsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_access_request_grants() -> CountAccessRequestGrantsStmt {
    CountAccessRequestGrantsStmt(
        "SELECT COUNT(*)::bigint AS count FROM access_request_grants WHERE request_id = $1::text",
        None,
    )
}
impl CountAccessRequestGrantsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct CountAccessRequestApprovalTeamsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn count_access_request_approval_teams() -> CountAccessRequestApprovalTeamsStmt {
    CountAccessRequestApprovalTeamsStmt(
        "SELECT COUNT(*)::bigint AS count FROM access_request_approval_teams WHERE request_id = $1::text",
        None,
    )
}
impl CountAccessRequestApprovalTeamsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct CreateAccessRequestEventStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_access_request_event() -> CreateAccessRequestEventStmt {
    CreateAccessRequestEventStmt(
        "INSERT INTO access_request_events (request_id, event_type, actor, comment) VALUES ( $1::text, $2::text, $3::text, NULLIF($4::text, '') )",
        None,
    )
}
impl CreateAccessRequestEventStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        event_type: &'a T2,
        actor: &'a T3,
        comment: &'a T4,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[request_id, event_type, actor, comment])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateAccessRequestEventParams<T1, T2, T3, T4>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateAccessRequestEventStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateAccessRequestEventParams<T1, T2, T3, T4>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.event_type,
            &params.actor,
            &params.comment,
        ))
    }
}
pub struct ListAccessRequestsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_requests() -> ListAccessRequestsStmt {
    ListAccessRequestsStmt(
        "SELECT requests.id, COALESCE(requests.template_id, '') AS template_id, COALESCE(requests.template_version, 0) AS template_version, requests.template_name, requests.requested_by, requests.requested_for, COALESCE(requests.routed_for_team, '') AS routed_for_team, requests.target_securable_type, requests.target_full_name, requests.reason, requests.status, requests.requested_duration_seconds, requests.submitted_at::text AS submitted_at, COALESCE(requests.decided_by, '') AS decided_by, COALESCE(requests.decided_at::text, '') AS decided_at, COALESCE(requests.decision_comment, '') AS decision_comment FROM access_requests requests WHERE ($1::text = '' OR requests.status = $1::text) AND ($2::text = '' OR requests.requested_by = $2::text) AND ($3::text = '' OR requests.requested_for = $3::text) AND ( $4::text = '' OR EXISTS ( SELECT 1 FROM access_request_approval_teams approval WHERE approval.request_id = requests.id AND approval.team = $4::text ) ) ORDER BY requests.submitted_at DESC, requests.id",
        None,
    )
}
impl ListAccessRequestsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        status: &'a T1,
        requested_by: &'a T2,
        requested_for: &'a T3,
        approval_team: &'a T4,
    ) -> ListAccessRequestsQuery<'c, 'a, 's, C, ListAccessRequests, 4> {
        ListAccessRequestsQuery {
            client,
            params: [status, requested_by, requested_for, approval_team],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAccessRequestsBorrowed, tokio_postgres::Error> {
                Ok(ListAccessRequestsBorrowed {
                    id: row.try_get(0)?,
                    template_id: row.try_get(1)?,
                    template_version: row.try_get(2)?,
                    template_name: row.try_get(3)?,
                    requested_by: row.try_get(4)?,
                    requested_for: row.try_get(5)?,
                    routed_for_team: row.try_get(6)?,
                    target_securable_type: row.try_get(7)?,
                    target_full_name: row.try_get(8)?,
                    reason: row.try_get(9)?,
                    status: row.try_get(10)?,
                    requested_duration_seconds: row.try_get(11)?,
                    submitted_at: row.try_get(12)?,
                    decided_by: row.try_get(13)?,
                    decided_at: row.try_get(14)?,
                    decision_comment: row.try_get(15)?,
                })
            },
            mapper: |it| ListAccessRequests::from(it),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListAccessRequestsParams<T1, T2, T3, T4>,
        ListAccessRequestsQuery<'c, 'a, 's, C, ListAccessRequests, 4>,
        C,
    > for ListAccessRequestsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListAccessRequestsParams<T1, T2, T3, T4>,
    ) -> ListAccessRequestsQuery<'c, 'a, 's, C, ListAccessRequests, 4> {
        self.bind(
            client,
            &params.status,
            &params.requested_by,
            &params.requested_for,
            &params.approval_team,
        )
    }
}
pub struct ListApprovedAccessRequestIdsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_approved_access_request_ids() -> ListApprovedAccessRequestIdsStmt {
    ListApprovedAccessRequestIdsStmt(
        "SELECT id FROM access_requests WHERE status = 'APPROVED' AND access_expires_at > NOW() ORDER BY decided_at, id",
        None,
    )
}
impl ListApprovedAccessRequestIdsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> StringQuery<'c, 'a, 's, C, String, 0> {
        StringQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct StartAccessRequestFulfillmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn start_access_request_fulfillment() -> StartAccessRequestFulfillmentStmt {
    StartAccessRequestFulfillmentStmt(
        "UPDATE access_requests SET status = 'FULFILLING' WHERE id = $1::text AND status = 'APPROVED' AND access_expires_at > NOW() RETURNING requested_for",
        None,
    )
}
impl StartAccessRequestFulfillmentStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct SetAccessRequestGrantStatusStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_access_request_grant_status() -> SetAccessRequestGrantStatusStmt {
    SetAccessRequestGrantStatusStmt(
        "UPDATE access_request_grants SET fulfillment_status = $1::text, fulfilled_at = CASE WHEN $1::text = 'GRANTED' THEN NOW() ELSE fulfilled_at END, failure_message = NULLIF($2::text, '') WHERE request_id = $3::text AND securable_type = $4::text AND full_name = $5::text AND privilege = $6::text",
        None,
    )
}
impl SetAccessRequestGrantStatusStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
        T6: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        status: &'a T1,
        failure_message: &'a T2,
        request_id: &'a T3,
        securable_type: &'a T4,
        full_name: &'a T5,
        privilege: &'a T6,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    status,
                    failure_message,
                    request_id,
                    securable_type,
                    full_name,
                    privilege,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetAccessRequestGrantStatusParams<T1, T2, T3, T4, T5, T6>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetAccessRequestGrantStatusStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetAccessRequestGrantStatusParams<T1, T2, T3, T4, T5, T6>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.status,
            &params.failure_message,
            &params.request_id,
            &params.securable_type,
            &params.full_name,
            &params.privilege,
        ))
    }
}
pub struct CompleteAccessRequestFulfillmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn complete_access_request_fulfillment() -> CompleteAccessRequestFulfillmentStmt {
    CompleteAccessRequestFulfillmentStmt(
        "UPDATE access_requests SET status = 'FULFILLED', fulfilled_at = NOW() WHERE id = $1::text AND status = 'FULFILLING'",
        None,
    )
}
impl CompleteAccessRequestFulfillmentStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[request_id]).await
    }
}
pub struct FailAccessRequestFulfillmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fail_access_request_fulfillment() -> FailAccessRequestFulfillmentStmt {
    FailAccessRequestFulfillmentStmt(
        "UPDATE access_requests SET status = 'FAILED' WHERE id = $1::text AND status = 'FULFILLING'",
        None,
    )
}
impl FailAccessRequestFulfillmentStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[request_id]).await
    }
}
pub struct FailPendingAccessRequestGrantsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fail_pending_access_request_grants() -> FailPendingAccessRequestGrantsStmt {
    FailPendingAccessRequestGrantsStmt(
        "UPDATE access_request_grants SET fulfillment_status = 'FAILED', failure_message = $1::text WHERE request_id = $2::text AND fulfillment_status = 'PENDING'",
        None,
    )
}
impl FailPendingAccessRequestGrantsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        failure_message: &'a T1,
        request_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[failure_message, request_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        FailPendingAccessRequestGrantsParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for FailPendingAccessRequestGrantsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a FailPendingAccessRequestGrantsParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.failure_message, &params.request_id))
    }
}
pub struct ListExpiredFulfilledAccessRequestsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_expired_fulfilled_access_requests() -> ListExpiredFulfilledAccessRequestsStmt {
    ListExpiredFulfilledAccessRequestsStmt(
        "SELECT id, requested_for FROM access_requests WHERE status = 'FULFILLED' AND access_expires_at <= NOW() ORDER BY access_expires_at, id",
        None,
    )
}
impl ListExpiredFulfilledAccessRequestsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> ListExpiredFulfilledAccessRequestsQuery<'c, 'a, 's, C, ListExpiredFulfilledAccessRequests, 0>
    {
        ListExpiredFulfilledAccessRequestsQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                ListExpiredFulfilledAccessRequestsBorrowed,
                tokio_postgres::Error,
            > {
                Ok(ListExpiredFulfilledAccessRequestsBorrowed {
                    id: row.try_get(0)?,
                    requested_for: row.try_get(1)?,
                })
            },
            mapper: |it| ListExpiredFulfilledAccessRequests::from(it),
        }
    }
}
pub struct ListAccessRequestsPendingRevocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_requests_pending_revocation() -> ListAccessRequestsPendingRevocationStmt {
    ListAccessRequestsPendingRevocationStmt(
        "SELECT DISTINCT requests.id, requests.requested_for FROM access_requests requests JOIN access_request_grants grants ON grants.request_id = requests.id WHERE requests.status IN ('REJECTED', 'FAILED') AND grants.fulfillment_status IN ('GRANTING', 'GRANTED', 'UNCHANGED') ORDER BY requests.id",
        None,
    )
}
impl ListAccessRequestsPendingRevocationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> ListAccessRequestsPendingRevocationQuery<
        'c,
        'a,
        's,
        C,
        ListAccessRequestsPendingRevocation,
        0,
    > {
        ListAccessRequestsPendingRevocationQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                ListAccessRequestsPendingRevocationBorrowed,
                tokio_postgres::Error,
            > {
                Ok(ListAccessRequestsPendingRevocationBorrowed {
                    id: row.try_get(0)?,
                    requested_for: row.try_get(1)?,
                })
            },
            mapper: |it| ListAccessRequestsPendingRevocation::from(it),
        }
    }
}
pub struct HasOtherActiveManagedGrantStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn has_other_active_managed_grant() -> HasOtherActiveManagedGrantStmt {
    HasOtherActiveManagedGrantStmt(
        "SELECT EXISTS ( SELECT 1 FROM access_request_grants grants JOIN access_requests requests ON requests.id = grants.request_id WHERE grants.request_id <> $1::text AND requests.requested_for = $2::text AND requests.status = 'FULFILLED' AND requests.access_expires_at > NOW() AND grants.securable_type = $3::text AND grants.full_name = $4::text AND grants.privilege = $5::text AND grants.fulfillment_status = 'GRANTED' ) AS active",
        None,
    )
}
impl HasOtherActiveManagedGrantStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        requested_for: &'a T2,
        securable_type: &'a T3,
        full_name: &'a T4,
        privilege: &'a T5,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 5> {
        BoolQuery {
            client,
            params: [
                request_id,
                requested_for,
                securable_type,
                full_name,
                privilege,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        HasOtherActiveManagedGrantParams<T1, T2, T3, T4, T5>,
        BoolQuery<'c, 'a, 's, C, bool, 5>,
        C,
    > for HasOtherActiveManagedGrantStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a HasOtherActiveManagedGrantParams<T1, T2, T3, T4, T5>,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 5> {
        self.bind(
            client,
            &params.request_id,
            &params.requested_for,
            &params.securable_type,
            &params.full_name,
            &params.privilege,
        )
    }
}
pub struct ExpireAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_access_request() -> ExpireAccessRequestStmt {
    ExpireAccessRequestStmt(
        "UPDATE access_requests SET status = 'EXPIRED' WHERE id = $1::text AND status = 'FULFILLED'",
        None,
    )
}
impl ExpireAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[request_id]).await
    }
}
pub struct ExpireUnfulfilledAccessRequestsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_unfulfilled_access_requests() -> ExpireUnfulfilledAccessRequestsStmt {
    ExpireUnfulfilledAccessRequestsStmt(
        "WITH expired AS ( UPDATE access_requests SET status = 'EXPIRED' WHERE status = 'APPROVED' AND access_expires_at <= NOW() RETURNING id ), events AS ( INSERT INTO access_request_events (request_id, event_type, actor, comment) SELECT id, 'EXPIRED', 'system:expiry', 'Requested access duration elapsed before fulfillment' FROM expired RETURNING request_id ) SELECT request_id FROM events ORDER BY request_id",
        None,
    )
}
impl ExpireUnfulfilledAccessRequestsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> StringQuery<'c, 'a, 's, C, String, 0> {
        StringQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct GetAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_access_request() -> GetAccessRequestStmt {
    GetAccessRequestStmt(
        "SELECT requests.id, COALESCE(requests.template_id, '') AS template_id, COALESCE(requests.template_version, 0) AS template_version, requests.template_name, requests.approval_strategy, requests.requested_by, requests.requested_for, COALESCE(requests.routed_for_team, '') AS routed_for_team, requests.target_securable_type, requests.target_full_name, COALESCE(requests.contract_id, '') AS contract_id, COALESCE(requests.contract_version, '') AS contract_version, COALESCE(requests.contract_document_hash, '') AS contract_document_hash, COALESCE(requests.purpose_code, '') AS purpose_code, requests.consumer_context, requests.contract_snapshot, requests.contract_evaluations, requests.contract_evaluation_status, COALESCE(requests.contract_evaluated_at::text, '') AS contract_evaluated_at, requests.reason, requests.status, requests.requested_duration_seconds, requests.submitted_at::text AS submitted_at, COALESCE(requests.decided_by, '') AS decided_by, COALESCE(requests.decided_at::text, '') AS decided_at, COALESCE(requests.decision_comment, '') AS decision_comment, COALESCE(requests.fulfilled_at::text, '') AS fulfilled_at, COALESCE(requests.access_expires_at::text, '') AS access_expires_at FROM access_requests requests WHERE requests.id = $1::text",
        None,
    )
}
impl GetAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> GetAccessRequestQuery<'c, 'a, 's, C, GetAccessRequest, 1> {
        GetAccessRequestQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetAccessRequestBorrowed, tokio_postgres::Error> {
                Ok(GetAccessRequestBorrowed {
                    id: row.try_get(0)?,
                    template_id: row.try_get(1)?,
                    template_version: row.try_get(2)?,
                    template_name: row.try_get(3)?,
                    approval_strategy: row.try_get(4)?,
                    requested_by: row.try_get(5)?,
                    requested_for: row.try_get(6)?,
                    routed_for_team: row.try_get(7)?,
                    target_securable_type: row.try_get(8)?,
                    target_full_name: row.try_get(9)?,
                    contract_id: row.try_get(10)?,
                    contract_version: row.try_get(11)?,
                    contract_document_hash: row.try_get(12)?,
                    purpose_code: row.try_get(13)?,
                    consumer_context: row.try_get(14)?,
                    contract_snapshot: row.try_get(15)?,
                    contract_evaluations: row.try_get(16)?,
                    contract_evaluation_status: row.try_get(17)?,
                    contract_evaluated_at: row.try_get(18)?,
                    reason: row.try_get(19)?,
                    status: row.try_get(20)?,
                    requested_duration_seconds: row.try_get(21)?,
                    submitted_at: row.try_get(22)?,
                    decided_by: row.try_get(23)?,
                    decided_at: row.try_get(24)?,
                    decision_comment: row.try_get(25)?,
                    fulfilled_at: row.try_get(26)?,
                    access_expires_at: row.try_get(27)?,
                })
            },
            mapper: |it| GetAccessRequest::from(it),
        }
    }
}
pub struct LockAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lock_access_request() -> LockAccessRequestStmt {
    LockAccessRequestStmt(
        "SELECT requests.status, requests.approval_strategy, requests.requested_for, COALESCE(requests.routed_for_team, '') AS routed_for_team, COALESCE(requests.decided_by, '') AS decided_by FROM access_requests requests WHERE requests.id = $1::text FOR UPDATE OF requests",
        None,
    )
}
impl LockAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> LockAccessRequestQuery<'c, 'a, 's, C, LockAccessRequest, 1> {
        LockAccessRequestQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<LockAccessRequestBorrowed, tokio_postgres::Error> {
                Ok(LockAccessRequestBorrowed {
                    status: row.try_get(0)?,
                    approval_strategy: row.try_get(1)?,
                    requested_for: row.try_get(2)?,
                    routed_for_team: row.try_get(3)?,
                    decided_by: row.try_get(4)?,
                })
            },
            mapper: |it| LockAccessRequest::from(it),
        }
    }
}
pub struct LockOwnedAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lock_owned_access_request() -> LockOwnedAccessRequestStmt {
    LockOwnedAccessRequestStmt(
        "SELECT requests.status, COALESCE(versions.document, '{}'::jsonb) AS contract_document FROM access_requests requests LEFT JOIN data_contract_versions versions ON versions.contract_id = requests.contract_id AND versions.version = requests.contract_version WHERE requests.id = $1::text AND requests.requested_by = $2::text FOR UPDATE OF requests",
        None,
    )
}
impl LockOwnedAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        requested_by: &'a T2,
    ) -> LockOwnedAccessRequestQuery<'c, 'a, 's, C, LockOwnedAccessRequest, 2> {
        LockOwnedAccessRequestQuery {
            client,
            params: [request_id, requested_by],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<LockOwnedAccessRequestBorrowed, tokio_postgres::Error> {
                Ok(LockOwnedAccessRequestBorrowed {
                    status: row.try_get(0)?,
                    contract_document: row.try_get(1)?,
                })
            },
            mapper: |it| LockOwnedAccessRequest::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        LockOwnedAccessRequestParams<T1, T2>,
        LockOwnedAccessRequestQuery<'c, 'a, 's, C, LockOwnedAccessRequest, 2>,
        C,
    > for LockOwnedAccessRequestStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a LockOwnedAccessRequestParams<T1, T2>,
    ) -> LockOwnedAccessRequestQuery<'c, 'a, 's, C, LockOwnedAccessRequest, 2> {
        self.bind(client, &params.request_id, &params.requested_by)
    }
}
pub struct ClearAccessRequestReviewsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn clear_access_request_reviews() -> ClearAccessRequestReviewsStmt {
    ClearAccessRequestReviewsStmt(
        "DELETE FROM access_request_reviews WHERE request_id = $1::text",
        None,
    )
}
impl ClearAccessRequestReviewsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[request_id]).await
    }
}
pub struct UpdateAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_access_request() -> UpdateAccessRequestStmt {
    UpdateAccessRequestStmt(
        "UPDATE access_requests SET reason = $1::text, requested_duration_seconds = $2::bigint WHERE id = $3::text",
        None,
    )
}
impl UpdateAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        reason: &'a T1,
        requested_duration_seconds: &'a i64,
        request_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[reason, requested_duration_seconds, request_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateAccessRequestParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateAccessRequestStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateAccessRequestParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.reason,
            &params.requested_duration_seconds,
            &params.request_id,
        ))
    }
}
pub struct CancelAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn cancel_access_request() -> CancelAccessRequestStmt {
    CancelAccessRequestStmt(
        "UPDATE access_requests SET status = 'CANCELLED', decided_by = $1::text, decided_at = NOW(), decision_comment = 'Request cancelled by requester' WHERE id = $2::text",
        None,
    )
}
impl CancelAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        requested_by: &'a T1,
        request_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[requested_by, request_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CancelAccessRequestParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CancelAccessRequestStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CancelAccessRequestParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.requested_by, &params.request_id))
    }
}
pub struct ListAccessRequestApprovalTeamsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_request_approval_teams() -> ListAccessRequestApprovalTeamsStmt {
    ListAccessRequestApprovalTeamsStmt(
        "SELECT teams.team, teams.required_approvals, ( SELECT COUNT(*)::int4 FROM access_request_reviews reviews WHERE reviews.request_id = teams.request_id AND reviews.approval_team = teams.team AND reviews.decision = 'APPROVED' ) AS approved_count, ( SELECT COUNT(*)::int4 FROM access_request_reviews reviews WHERE reviews.request_id = teams.request_id AND reviews.approval_team = teams.team AND reviews.decision = 'REJECTED' ) AS rejected_count FROM access_request_approval_teams teams WHERE teams.request_id = $1::text ORDER BY team",
        None,
    )
}
impl ListAccessRequestApprovalTeamsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> ListAccessRequestApprovalTeamsQuery<'c, 'a, 's, C, ListAccessRequestApprovalTeams, 1> {
        ListAccessRequestApprovalTeamsQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                ListAccessRequestApprovalTeamsBorrowed,
                tokio_postgres::Error,
            > {
                Ok(ListAccessRequestApprovalTeamsBorrowed {
                    team: row.try_get(0)?,
                    required_approvals: row.try_get(1)?,
                    approved_count: row.try_get(2)?,
                    rejected_count: row.try_get(3)?,
                })
            },
            mapper: |it| ListAccessRequestApprovalTeams::from(it),
        }
    }
}
pub struct ListAccessRequestGrantsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_request_grants() -> ListAccessRequestGrantsStmt {
    ListAccessRequestGrantsStmt(
        "SELECT securable_type, full_name, privilege, fulfillment_status FROM access_request_grants WHERE request_id = $1::text ORDER BY securable_type, full_name, privilege",
        None,
    )
}
impl ListAccessRequestGrantsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> ListAccessRequestGrantsQuery<'c, 'a, 's, C, ListAccessRequestGrants, 1> {
        ListAccessRequestGrantsQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAccessRequestGrantsBorrowed, tokio_postgres::Error> {
                Ok(ListAccessRequestGrantsBorrowed {
                    securable_type: row.try_get(0)?,
                    full_name: row.try_get(1)?,
                    privilege: row.try_get(2)?,
                    fulfillment_status: row.try_get(3)?,
                })
            },
            mapper: |it| ListAccessRequestGrants::from(it),
        }
    }
}
pub struct ListAccessRequestGuardrailsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_request_guardrails() -> ListAccessRequestGuardrailsStmt {
    ListAccessRequestGuardrailsStmt(
        "SELECT guardrail_key, name, COALESCE(description, '') AS description, enforcement, configuration, evaluation_status FROM access_request_guardrails WHERE request_id = $1::text ORDER BY guardrail_key",
        None,
    )
}
impl ListAccessRequestGuardrailsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> ListAccessRequestGuardrailsQuery<'c, 'a, 's, C, ListAccessRequestGuardrails, 1> {
        ListAccessRequestGuardrailsQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAccessRequestGuardrailsBorrowed, tokio_postgres::Error> {
                Ok(ListAccessRequestGuardrailsBorrowed {
                    guardrail_key: row.try_get(0)?,
                    name: row.try_get(1)?,
                    description: row.try_get(2)?,
                    enforcement: row.try_get(3)?,
                    configuration: row.try_get(4)?,
                    evaluation_status: row.try_get(5)?,
                })
            },
            mapper: |it| ListAccessRequestGuardrails::from(it),
        }
    }
}
pub struct ListAccessRequestReviewsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_request_reviews() -> ListAccessRequestReviewsStmt {
    ListAccessRequestReviewsStmt(
        "SELECT approval_team, reviewer, decision, COALESCE(comment, '') AS comment, created_at::text AS created_at FROM access_request_reviews WHERE request_id = $1::text ORDER BY created_at, id",
        None,
    )
}
impl ListAccessRequestReviewsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> ListAccessRequestReviewsQuery<'c, 'a, 's, C, ListAccessRequestReviews, 1> {
        ListAccessRequestReviewsQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAccessRequestReviewsBorrowed, tokio_postgres::Error> {
                Ok(ListAccessRequestReviewsBorrowed {
                    approval_team: row.try_get(0)?,
                    reviewer: row.try_get(1)?,
                    decision: row.try_get(2)?,
                    comment: row.try_get(3)?,
                    created_at: row.try_get(4)?,
                })
            },
            mapper: |it| ListAccessRequestReviews::from(it),
        }
    }
}
pub struct ListAccessRequestEventsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_access_request_events() -> ListAccessRequestEventsStmt {
    ListAccessRequestEventsStmt(
        "SELECT event_type, actor, COALESCE(comment, '') AS comment, created_at::text AS created_at FROM access_request_events WHERE request_id = $1::text ORDER BY created_at, id",
        None,
    )
}
impl ListAccessRequestEventsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> ListAccessRequestEventsQuery<'c, 'a, 's, C, ListAccessRequestEvents, 1> {
        ListAccessRequestEventsQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListAccessRequestEventsBorrowed, tokio_postgres::Error> {
                Ok(ListAccessRequestEventsBorrowed {
                    event_type: row.try_get(0)?,
                    actor: row.try_get(1)?,
                    comment: row.try_get(2)?,
                    created_at: row.try_get(3)?,
                })
            },
            mapper: |it| ListAccessRequestEvents::from(it),
        }
    }
}
pub struct IsAccessRequestApprovalTeamStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn is_access_request_approval_team() -> IsAccessRequestApprovalTeamStmt {
    IsAccessRequestApprovalTeamStmt(
        "SELECT EXISTS ( SELECT 1 FROM access_request_approval_teams WHERE request_id = $1::text AND team = $2::text ) AS assigned",
        None,
    )
}
impl IsAccessRequestApprovalTeamStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        approval_team: &'a T2,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 2> {
        BoolQuery {
            client,
            params: [request_id, approval_team],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        IsAccessRequestApprovalTeamParams<T1, T2>,
        BoolQuery<'c, 'a, 's, C, bool, 2>,
        C,
    > for IsAccessRequestApprovalTeamStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a IsAccessRequestApprovalTeamParams<T1, T2>,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 2> {
        self.bind(client, &params.request_id, &params.approval_team)
    }
}
pub struct CreateAccessRequestReviewStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_access_request_review() -> CreateAccessRequestReviewStmt {
    CreateAccessRequestReviewStmt(
        "INSERT INTO access_request_reviews ( request_id, approval_team, reviewer, decision, comment ) VALUES ( $1::text, $2::text, $3::text, $4::text, NULLIF($5::text, '') ) ON CONFLICT (request_id, reviewer) DO UPDATE SET approval_team = EXCLUDED.approval_team, decision = EXCLUDED.decision, comment = EXCLUDED.comment, created_at = NOW()",
        None,
    )
}
impl CreateAccessRequestReviewStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
        approval_team: &'a T2,
        reviewer: &'a T3,
        decision: &'a T4,
        comment: &'a T5,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[request_id, approval_team, reviewer, decision, comment],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateAccessRequestReviewParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateAccessRequestReviewStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateAccessRequestReviewParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.request_id,
            &params.approval_team,
            &params.reviewer,
            &params.decision,
            &params.comment,
        ))
    }
}
pub struct IsAccessRequestFullyApprovedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn is_access_request_fully_approved() -> IsAccessRequestFullyApprovedStmt {
    IsAccessRequestFullyApprovedStmt(
        "SELECT CASE requests.approval_strategy WHEN 'ALL' THEN NOT EXISTS ( SELECT 1 FROM access_request_approval_teams teams WHERE teams.request_id = requests.id AND ( SELECT COUNT(*) FROM access_request_reviews reviews WHERE reviews.request_id = requests.id AND reviews.approval_team = teams.team AND reviews.decision = 'APPROVED' ) < teams.required_approvals ) ELSE EXISTS ( SELECT 1 FROM access_request_approval_teams teams WHERE teams.request_id = requests.id AND ( SELECT COUNT(*) FROM access_request_reviews reviews WHERE reviews.request_id = requests.id AND reviews.approval_team = teams.team AND reviews.decision = 'APPROVED' ) >= teams.required_approvals ) END AS approved FROM access_requests requests WHERE requests.id = $1::text",
        None,
    )
}
impl IsAccessRequestFullyApprovedStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        request_id: &'a T1,
    ) -> BoolQuery<'c, 'a, 's, C, bool, 1> {
        BoolQuery {
            client,
            params: [request_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct ApproveAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn approve_access_request() -> ApproveAccessRequestStmt {
    ApproveAccessRequestStmt(
        "UPDATE access_requests SET status = 'APPROVED', decided_by = $1::text, decided_at = NOW(), decision_comment = NULLIF($2::text, ''), access_expires_at = NOW() + (requested_duration_seconds * INTERVAL '1 second') WHERE id = $3::text",
        None,
    )
}
impl ApproveAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        reviewer: &'a T1,
        comment: &'a T2,
        request_id: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[reviewer, comment, request_id])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        ApproveAccessRequestParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for ApproveAccessRequestStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a ApproveAccessRequestParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.reviewer,
            &params.comment,
            &params.request_id,
        ))
    }
}
pub struct RejectAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn reject_access_request() -> RejectAccessRequestStmt {
    RejectAccessRequestStmt(
        "UPDATE access_requests SET status = 'REJECTED', decided_by = $1::text, decided_at = NOW(), decision_comment = NULLIF($2::text, ''), access_expires_at = CASE WHEN access_expires_at IS NULL THEN NOW() ELSE LEAST(access_expires_at, NOW()) END WHERE id = $3::text",
        None,
    )
}
impl RejectAccessRequestStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        reviewer: &'a T1,
        comment: &'a T2,
        request_id: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[reviewer, comment, request_id])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RejectAccessRequestParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RejectAccessRequestStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RejectAccessRequestParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.reviewer,
            &params.comment,
            &params.request_id,
        ))
    }
}
