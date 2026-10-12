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
    pub reason: T9,
    pub auto_approve: bool,
    pub requested_duration_seconds: i64,
    pub idempotency_key: T10,
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
pub struct RevokeAccessRequestParams<T1: crate::StringSql, T2: crate::StringSql> {
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
}
pub struct LockAccessRequestBorrowed<'a> {
    pub status: &'a str,
    pub approval_strategy: &'a str,
}
impl<'a> From<LockAccessRequestBorrowed<'a>> for LockAccessRequest {
    fn from(
        LockAccessRequestBorrowed {
            status,
            approval_strategy,
        }: LockAccessRequestBorrowed<'a>,
    ) -> Self {
        Self {
            status: status.into(),
            approval_strategy: approval_strategy.into(),
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
        "INSERT INTO access_requests ( template_id, template_version, template_name, approval_strategy, requested_by, requested_for, routed_for_team, target_securable_type, target_full_name, reason, status, requested_duration_seconds, idempotency_key, decided_by, decided_at, decision_comment, access_expires_at ) VALUES ( NULLIF($1::text, ''), CASE WHEN $1::text = '' THEN NULL ELSE $2::integer END, $3::text, $4::text, $5::text, $6::text, $7::text, $8::text, $9::text, $10::text, CASE WHEN $11::boolean THEN 'APPROVED' ELSE 'AWAITING_APPROVAL' END, $12::bigint, NULLIF($13::text, ''), CASE WHEN $11::boolean THEN 'system:auto-approve' END, CASE WHEN $11::boolean THEN NOW() END, CASE WHEN $11::boolean THEN 'Automatically approved by policy template' END, CASE WHEN $11::boolean THEN NOW() + ($12::bigint * INTERVAL '1 second') END ) RETURNING id",
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
        reason: &'a T9,
        auto_approve: &'a bool,
        requested_duration_seconds: &'a i64,
        idempotency_key: &'a T10,
    ) -> StringQuery<'c, 'a, 's, C, String, 13> {
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
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateAccessRequestParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>,
        StringQuery<'c, 'a, 's, C, String, 13>,
        C,
    > for CreateAccessRequestStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateAccessRequestParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>,
    ) -> StringQuery<'c, 'a, 's, C, String, 13> {
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
            &params.reason,
            &params.auto_approve,
            &params.requested_duration_seconds,
            &params.idempotency_key,
        )
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
pub struct ExpireAccessRequestsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_access_requests() -> ExpireAccessRequestsStmt {
    ExpireAccessRequestsStmt(
        "WITH expired AS ( UPDATE access_requests SET status = 'EXPIRED' WHERE status IN ('APPROVED', 'FULFILLED') AND access_expires_at <= NOW() RETURNING id ) INSERT INTO access_request_events (request_id, event_type, actor, comment) SELECT id, 'EXPIRED', 'system:expiry', 'Requested access duration elapsed' FROM expired",
        None,
    )
}
impl ExpireAccessRequestsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct GetAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_access_request() -> GetAccessRequestStmt {
    GetAccessRequestStmt(
        "SELECT requests.id, COALESCE(requests.template_id, '') AS template_id, COALESCE(requests.template_version, 0) AS template_version, requests.template_name, requests.approval_strategy, requests.requested_by, requests.requested_for, COALESCE(requests.routed_for_team, '') AS routed_for_team, requests.target_securable_type, requests.target_full_name, requests.reason, requests.status, requests.requested_duration_seconds, requests.submitted_at::text AS submitted_at, COALESCE(requests.decided_by, '') AS decided_by, COALESCE(requests.decided_at::text, '') AS decided_at, COALESCE(requests.decision_comment, '') AS decision_comment, COALESCE(requests.fulfilled_at::text, '') AS fulfilled_at, COALESCE(requests.access_expires_at::text, '') AS access_expires_at FROM access_requests requests WHERE requests.id = $1::text",
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
                    reason: row.try_get(10)?,
                    status: row.try_get(11)?,
                    requested_duration_seconds: row.try_get(12)?,
                    submitted_at: row.try_get(13)?,
                    decided_by: row.try_get(14)?,
                    decided_at: row.try_get(15)?,
                    decision_comment: row.try_get(16)?,
                    fulfilled_at: row.try_get(17)?,
                    access_expires_at: row.try_get(18)?,
                })
            },
            mapper: |it| GetAccessRequest::from(it),
        }
    }
}
pub struct LockAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lock_access_request() -> LockAccessRequestStmt {
    LockAccessRequestStmt(
        "SELECT requests.status, requests.approval_strategy FROM access_requests requests WHERE requests.id = $1::text FOR UPDATE OF requests",
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
                })
            },
            mapper: |it| LockAccessRequest::from(it),
        }
    }
}
pub struct LockOwnedAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn lock_owned_access_request() -> LockOwnedAccessRequestStmt {
    LockOwnedAccessRequestStmt(
        "SELECT status FROM access_requests WHERE id = $1::text AND requested_by = $2::text FOR UPDATE",
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
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [request_id, requested_by],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        LockOwnedAccessRequestParams<T1, T2>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for LockOwnedAccessRequestStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a LockOwnedAccessRequestParams<T1, T2>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
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
pub struct RevokeAccessRequestStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_access_request() -> RevokeAccessRequestStmt {
    RevokeAccessRequestStmt(
        "UPDATE access_requests SET status = 'CANCELLED', decided_by = $1::text, decided_at = NOW(), decision_comment = 'Access revoked by requester', access_expires_at = CASE WHEN access_expires_at IS NULL THEN NOW() ELSE LEAST(access_expires_at, NOW()) END WHERE id = $2::text",
        None,
    )
}
impl RevokeAccessRequestStmt {
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
        RevokeAccessRequestParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RevokeAccessRequestStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RevokeAccessRequestParams<T1, T2>,
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
        "INSERT INTO access_request_reviews ( request_id, approval_team, reviewer, decision, comment ) VALUES ( $1::text, $2::text, $3::text, $4::text, NULLIF($5::text, '') )",
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
        "UPDATE access_requests SET status = 'REJECTED', decided_by = $1::text, decided_at = NOW(), decision_comment = NULLIF($2::text, '') WHERE id = $3::text",
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
