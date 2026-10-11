// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreatePolicyTemplateParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub key: T1,
    pub created_by: T2,
}
#[derive(Debug)]
pub struct CreatePolicyTemplateVersionParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub template_id: T1,
    pub name: T2,
    pub description: T3,
    pub approval_strategy: T4,
    pub default_duration_seconds: i64,
    pub max_duration_seconds: i64,
    pub created_by: T5,
}
#[derive(Debug)]
pub struct CreatePolicyTemplateGrantParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub template_id: T1,
    pub securable_type: T2,
    pub target_kind: T3,
    pub target_full_name: T4,
    pub privilege: T5,
}
#[derive(Debug)]
pub struct CreatePolicyTemplateAffectedTeamParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub template_id: T1,
    pub team: T2,
}
#[derive(Debug)]
pub struct CreatePolicyTemplateApprovalTeamParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub template_id: T1,
    pub team: T2,
    pub required_approvals: i32,
}
#[derive(Debug)]
pub struct CreatePolicyTemplateGuardrailParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::JsonSql,
    T7: crate::StringSql,
> {
    pub template_id: T1,
    pub guardrail_key: T2,
    pub name: T3,
    pub description: T4,
    pub enforcement: T5,
    pub configuration: T6,
    pub created_by: T7,
}
#[derive(Debug)]
pub struct ListPolicyTemplateGrantsParams<T1: crate::StringSql> {
    pub template_id: T1,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct ListPolicyTemplateAffectedTeamsParams<T1: crate::StringSql> {
    pub template_id: T1,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct ListPolicyTemplateApprovalTeamsParams<T1: crate::StringSql> {
    pub template_id: T1,
    pub template_version: i32,
}
#[derive(Debug)]
pub struct ListPolicyTemplateGuardrailsParams<T1: crate::StringSql> {
    pub template_id: T1,
    pub template_version: i32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListPolicyTemplates {
    pub id: String,
    pub key: String,
    pub version: i32,
    pub name: String,
    pub description: String,
    pub approval_strategy: String,
    pub default_duration_seconds: i64,
    pub max_duration_seconds: i64,
    pub created_by: String,
    pub created_at: String,
}
pub struct ListPolicyTemplatesBorrowed<'a> {
    pub id: &'a str,
    pub key: &'a str,
    pub version: i32,
    pub name: &'a str,
    pub description: &'a str,
    pub approval_strategy: &'a str,
    pub default_duration_seconds: i64,
    pub max_duration_seconds: i64,
    pub created_by: &'a str,
    pub created_at: &'a str,
}
impl<'a> From<ListPolicyTemplatesBorrowed<'a>> for ListPolicyTemplates {
    fn from(
        ListPolicyTemplatesBorrowed {
            id,
            key,
            version,
            name,
            description,
            approval_strategy,
            default_duration_seconds,
            max_duration_seconds,
            created_by,
            created_at,
        }: ListPolicyTemplatesBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            version,
            name: name.into(),
            description: description.into(),
            approval_strategy: approval_strategy.into(),
            default_duration_seconds,
            max_duration_seconds,
            created_by: created_by.into(),
            created_at: created_at.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetPolicyTemplate {
    pub id: String,
    pub key: String,
    pub version: i32,
    pub name: String,
    pub description: String,
    pub status: String,
    pub approval_strategy: String,
    pub blast_radius_required: bool,
    pub guardrail_review_required: bool,
    pub default_duration_seconds: i64,
    pub max_duration_seconds: i64,
    pub created_by: String,
    pub created_at: String,
    pub published_by: String,
    pub published_at: String,
}
pub struct GetPolicyTemplateBorrowed<'a> {
    pub id: &'a str,
    pub key: &'a str,
    pub version: i32,
    pub name: &'a str,
    pub description: &'a str,
    pub status: &'a str,
    pub approval_strategy: &'a str,
    pub blast_radius_required: bool,
    pub guardrail_review_required: bool,
    pub default_duration_seconds: i64,
    pub max_duration_seconds: i64,
    pub created_by: &'a str,
    pub created_at: &'a str,
    pub published_by: &'a str,
    pub published_at: &'a str,
}
impl<'a> From<GetPolicyTemplateBorrowed<'a>> for GetPolicyTemplate {
    fn from(
        GetPolicyTemplateBorrowed {
            id,
            key,
            version,
            name,
            description,
            status,
            approval_strategy,
            blast_radius_required,
            guardrail_review_required,
            default_duration_seconds,
            max_duration_seconds,
            created_by,
            created_at,
            published_by,
            published_at,
        }: GetPolicyTemplateBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            version,
            name: name.into(),
            description: description.into(),
            status: status.into(),
            approval_strategy: approval_strategy.into(),
            blast_radius_required,
            guardrail_review_required,
            default_duration_seconds,
            max_duration_seconds,
            created_by: created_by.into(),
            created_at: created_at.into(),
            published_by: published_by.into(),
            published_at: published_at.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListPolicyTemplateGrants {
    pub securable_type: String,
    pub target_kind: String,
    pub target_full_name: String,
    pub privilege: String,
}
pub struct ListPolicyTemplateGrantsBorrowed<'a> {
    pub securable_type: &'a str,
    pub target_kind: &'a str,
    pub target_full_name: &'a str,
    pub privilege: &'a str,
}
impl<'a> From<ListPolicyTemplateGrantsBorrowed<'a>> for ListPolicyTemplateGrants {
    fn from(
        ListPolicyTemplateGrantsBorrowed {
            securable_type,
            target_kind,
            target_full_name,
            privilege,
        }: ListPolicyTemplateGrantsBorrowed<'a>,
    ) -> Self {
        Self {
            securable_type: securable_type.into(),
            target_kind: target_kind.into(),
            target_full_name: target_full_name.into(),
            privilege: privilege.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListPolicyTemplateApprovalTeams {
    pub team: String,
    pub required_approvals: i32,
}
pub struct ListPolicyTemplateApprovalTeamsBorrowed<'a> {
    pub team: &'a str,
    pub required_approvals: i32,
}
impl<'a> From<ListPolicyTemplateApprovalTeamsBorrowed<'a>> for ListPolicyTemplateApprovalTeams {
    fn from(
        ListPolicyTemplateApprovalTeamsBorrowed {
            team,
            required_approvals,
        }: ListPolicyTemplateApprovalTeamsBorrowed<'a>,
    ) -> Self {
        Self {
            team: team.into(),
            required_approvals,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ListPolicyTemplateGuardrails {
    pub guardrail_key: String,
    pub name: String,
    pub description: String,
    pub enforcement: String,
    pub configuration: serde_json::Value,
}
pub struct ListPolicyTemplateGuardrailsBorrowed<'a> {
    pub guardrail_key: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub enforcement: &'a str,
    pub configuration: postgres_types::Json<&'a serde_json::value::RawValue>,
}
impl<'a> From<ListPolicyTemplateGuardrailsBorrowed<'a>> for ListPolicyTemplateGuardrails {
    fn from(
        ListPolicyTemplateGuardrailsBorrowed {
            guardrail_key,
            name,
            description,
            enforcement,
            configuration,
        }: ListPolicyTemplateGuardrailsBorrowed<'a>,
    ) -> Self {
        Self {
            guardrail_key: guardrail_key.into(),
            name: name.into(),
            description: description.into(),
            enforcement: enforcement.into(),
            configuration: serde_json::from_str(configuration.0.get()).unwrap(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct ListPolicyTemplatesQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListPolicyTemplatesBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPolicyTemplatesBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPolicyTemplatesQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPolicyTemplatesBorrowed) -> R,
    ) -> ListPolicyTemplatesQuery<'c, 'a, 's, C, R, N> {
        ListPolicyTemplatesQuery {
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
pub struct GetPolicyTemplateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetPolicyTemplateBorrowed, tokio_postgres::Error>,
    mapper: fn(GetPolicyTemplateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetPolicyTemplateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetPolicyTemplateBorrowed) -> R,
    ) -> GetPolicyTemplateQuery<'c, 'a, 's, C, R, N> {
        GetPolicyTemplateQuery {
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
pub struct ListPolicyTemplateGrantsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListPolicyTemplateGrantsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPolicyTemplateGrantsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPolicyTemplateGrantsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPolicyTemplateGrantsBorrowed) -> R,
    ) -> ListPolicyTemplateGrantsQuery<'c, 'a, 's, C, R, N> {
        ListPolicyTemplateGrantsQuery {
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
pub struct ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<ListPolicyTemplateApprovalTeamsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPolicyTemplateApprovalTeamsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPolicyTemplateApprovalTeamsBorrowed) -> R,
    ) -> ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C, R, N> {
        ListPolicyTemplateApprovalTeamsQuery {
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
pub struct ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<ListPolicyTemplateGuardrailsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPolicyTemplateGuardrailsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPolicyTemplateGuardrailsBorrowed) -> R,
    ) -> ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C, R, N> {
        ListPolicyTemplateGuardrailsQuery {
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
pub struct CreatePolicyTemplateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template() -> CreatePolicyTemplateStmt {
    CreatePolicyTemplateStmt(
        "INSERT INTO policy_templates (key, created_by) VALUES ($1::text, $2::text) RETURNING id",
        None,
    )
}
impl CreatePolicyTemplateStmt {
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
        key: &'a T1,
        created_by: &'a T2,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [key, created_by],
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
        CreatePolicyTemplateParams<T1, T2>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for CreatePolicyTemplateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreatePolicyTemplateParams<T1, T2>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.key, &params.created_by)
    }
}
pub struct CreatePolicyTemplateVersionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template_version() -> CreatePolicyTemplateVersionStmt {
    CreatePolicyTemplateVersionStmt(
        "INSERT INTO policy_template_versions ( template_id, version, name, description, status, approval_strategy, blast_radius_required, guardrail_review_required, default_duration_seconds, max_duration_seconds, created_by, published_by, published_at ) VALUES ( $1::text, 1, $2::text, NULLIF($3::text, ''), 'PUBLISHED', $4::text, FALSE, FALSE, NULLIF($5::bigint, 0), NULLIF($6::bigint, 0), $7::text, $7::text, NOW() )",
        None,
    )
}
impl CreatePolicyTemplateVersionStmt {
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
        template_id: &'a T1,
        name: &'a T2,
        description: &'a T3,
        approval_strategy: &'a T4,
        default_duration_seconds: &'a i64,
        max_duration_seconds: &'a i64,
        created_by: &'a T5,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    template_id,
                    name,
                    description,
                    approval_strategy,
                    default_duration_seconds,
                    max_duration_seconds,
                    created_by,
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreatePolicyTemplateVersionParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreatePolicyTemplateVersionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreatePolicyTemplateVersionParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.template_id,
            &params.name,
            &params.description,
            &params.approval_strategy,
            &params.default_duration_seconds,
            &params.max_duration_seconds,
            &params.created_by,
        ))
    }
}
pub struct CreatePolicyTemplateGrantStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template_grant() -> CreatePolicyTemplateGrantStmt {
    CreatePolicyTemplateGrantStmt(
        "INSERT INTO policy_template_grants ( template_id, template_version, securable_type, target_kind, target_full_name, privilege ) VALUES ( $1::text, 1, $2::text, $3::text, $4::text, $5::text )",
        None,
    )
}
impl CreatePolicyTemplateGrantStmt {
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
        template_id: &'a T1,
        securable_type: &'a T2,
        target_kind: &'a T3,
        target_full_name: &'a T4,
        privilege: &'a T5,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    template_id,
                    securable_type,
                    target_kind,
                    target_full_name,
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreatePolicyTemplateGrantParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreatePolicyTemplateGrantStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreatePolicyTemplateGrantParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.template_id,
            &params.securable_type,
            &params.target_kind,
            &params.target_full_name,
            &params.privilege,
        ))
    }
}
pub struct CreatePolicyTemplateAffectedTeamStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template_affected_team() -> CreatePolicyTemplateAffectedTeamStmt {
    CreatePolicyTemplateAffectedTeamStmt(
        "INSERT INTO policy_template_affected_teams (template_id, template_version, team) VALUES ($1::text, 1, $2::text)",
        None,
    )
}
impl CreatePolicyTemplateAffectedTeamStmt {
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
        template_id: &'a T1,
        team: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[template_id, team]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreatePolicyTemplateAffectedTeamParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreatePolicyTemplateAffectedTeamStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreatePolicyTemplateAffectedTeamParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.template_id, &params.team))
    }
}
pub struct CreatePolicyTemplateApprovalTeamStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template_approval_team() -> CreatePolicyTemplateApprovalTeamStmt {
    CreatePolicyTemplateApprovalTeamStmt(
        "INSERT INTO policy_template_approval_teams ( template_id, template_version, team, required_approvals ) VALUES ($1::text, 1, $2::text, $3)",
        None,
    )
}
impl CreatePolicyTemplateApprovalTeamStmt {
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
        template_id: &'a T1,
        team: &'a T2,
        required_approvals: &'a i32,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[template_id, team, required_approvals])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreatePolicyTemplateApprovalTeamParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreatePolicyTemplateApprovalTeamStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreatePolicyTemplateApprovalTeamParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.template_id,
            &params.team,
            &params.required_approvals,
        ))
    }
}
pub struct CreatePolicyTemplateGuardrailStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_policy_template_guardrail() -> CreatePolicyTemplateGuardrailStmt {
    CreatePolicyTemplateGuardrailStmt(
        "INSERT INTO policy_template_guardrails ( template_id, template_version, guardrail_key, name, description, enforcement, configuration, created_by ) VALUES ( $1::text, 1, $2::text, $3::text, NULLIF($4::text, ''), $5::text, $6, $7::text )",
        None,
    )
}
impl CreatePolicyTemplateGuardrailStmt {
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
        T6: crate::JsonSql,
        T7: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        template_id: &'a T1,
        guardrail_key: &'a T2,
        name: &'a T3,
        description: &'a T4,
        enforcement: &'a T5,
        configuration: &'a T6,
        created_by: &'a T7,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    template_id,
                    guardrail_key,
                    name,
                    description,
                    enforcement,
                    configuration,
                    created_by,
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
    T6: crate::JsonSql,
    T7: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreatePolicyTemplateGuardrailParams<T1, T2, T3, T4, T5, T6, T7>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreatePolicyTemplateGuardrailStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreatePolicyTemplateGuardrailParams<T1, T2, T3, T4, T5, T6, T7>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.template_id,
            &params.guardrail_key,
            &params.name,
            &params.description,
            &params.enforcement,
            &params.configuration,
            &params.created_by,
        ))
    }
}
pub struct ListPolicyTemplatesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_policy_templates() -> ListPolicyTemplatesStmt {
    ListPolicyTemplatesStmt(
        "SELECT templates.id, templates.key, versions.version, versions.name, COALESCE(versions.description, '') AS description, versions.approval_strategy, COALESCE(versions.default_duration_seconds, 0) AS default_duration_seconds, COALESCE(versions.max_duration_seconds, 0) AS max_duration_seconds, templates.created_by, templates.created_at::text AS created_at FROM policy_templates templates JOIN policy_template_versions versions ON versions.template_id = templates.id AND versions.status = 'PUBLISHED' WHERE ( $1::text = '' OR EXISTS ( SELECT 1 FROM policy_template_affected_teams affected WHERE affected.template_id = versions.template_id AND affected.template_version = versions.version AND affected.team = $1::text ) ) ORDER BY templates.key",
        None,
    )
}
impl ListPolicyTemplatesStmt {
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
        affected_team: &'a T1,
    ) -> ListPolicyTemplatesQuery<'c, 'a, 's, C, ListPolicyTemplates, 1> {
        ListPolicyTemplatesQuery {
            client,
            params: [affected_team],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListPolicyTemplatesBorrowed, tokio_postgres::Error> {
                Ok(ListPolicyTemplatesBorrowed {
                    id: row.try_get(0)?,
                    key: row.try_get(1)?,
                    version: row.try_get(2)?,
                    name: row.try_get(3)?,
                    description: row.try_get(4)?,
                    approval_strategy: row.try_get(5)?,
                    default_duration_seconds: row.try_get(6)?,
                    max_duration_seconds: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    created_at: row.try_get(9)?,
                })
            },
            mapper: |it| ListPolicyTemplates::from(it),
        }
    }
}
pub struct GetPolicyTemplateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_policy_template() -> GetPolicyTemplateStmt {
    GetPolicyTemplateStmt(
        "SELECT templates.id, templates.key, versions.version, versions.name, COALESCE(versions.description, '') AS description, versions.status, versions.approval_strategy, versions.blast_radius_required, versions.guardrail_review_required, COALESCE(versions.default_duration_seconds, 0) AS default_duration_seconds, COALESCE(versions.max_duration_seconds, 0) AS max_duration_seconds, templates.created_by, templates.created_at::text AS created_at, COALESCE(versions.published_by, '') AS published_by, COALESCE(versions.published_at::text, '') AS published_at FROM policy_templates templates JOIN policy_template_versions versions ON versions.template_id = templates.id WHERE templates.id = $1::text ORDER BY versions.version DESC LIMIT 1",
        None,
    )
}
impl GetPolicyTemplateStmt {
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
        template_id: &'a T1,
    ) -> GetPolicyTemplateQuery<'c, 'a, 's, C, GetPolicyTemplate, 1> {
        GetPolicyTemplateQuery {
            client,
            params: [template_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetPolicyTemplateBorrowed, tokio_postgres::Error> {
                Ok(GetPolicyTemplateBorrowed {
                    id: row.try_get(0)?,
                    key: row.try_get(1)?,
                    version: row.try_get(2)?,
                    name: row.try_get(3)?,
                    description: row.try_get(4)?,
                    status: row.try_get(5)?,
                    approval_strategy: row.try_get(6)?,
                    blast_radius_required: row.try_get(7)?,
                    guardrail_review_required: row.try_get(8)?,
                    default_duration_seconds: row.try_get(9)?,
                    max_duration_seconds: row.try_get(10)?,
                    created_by: row.try_get(11)?,
                    created_at: row.try_get(12)?,
                    published_by: row.try_get(13)?,
                    published_at: row.try_get(14)?,
                })
            },
            mapper: |it| GetPolicyTemplate::from(it),
        }
    }
}
pub struct ListPolicyTemplateGrantsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_policy_template_grants() -> ListPolicyTemplateGrantsStmt {
    ListPolicyTemplateGrantsStmt(
        "SELECT securable_type, target_kind, target_full_name, privilege FROM policy_template_grants WHERE template_id = $1::text AND template_version = $2 ORDER BY securable_type, target_kind, target_full_name, privilege",
        None,
    )
}
impl ListPolicyTemplateGrantsStmt {
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
        template_id: &'a T1,
        template_version: &'a i32,
    ) -> ListPolicyTemplateGrantsQuery<'c, 'a, 's, C, ListPolicyTemplateGrants, 2> {
        ListPolicyTemplateGrantsQuery {
            client,
            params: [template_id, template_version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListPolicyTemplateGrantsBorrowed, tokio_postgres::Error> {
                Ok(ListPolicyTemplateGrantsBorrowed {
                    securable_type: row.try_get(0)?,
                    target_kind: row.try_get(1)?,
                    target_full_name: row.try_get(2)?,
                    privilege: row.try_get(3)?,
                })
            },
            mapper: |it| ListPolicyTemplateGrants::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListPolicyTemplateGrantsParams<T1>,
        ListPolicyTemplateGrantsQuery<'c, 'a, 's, C, ListPolicyTemplateGrants, 2>,
        C,
    > for ListPolicyTemplateGrantsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListPolicyTemplateGrantsParams<T1>,
    ) -> ListPolicyTemplateGrantsQuery<'c, 'a, 's, C, ListPolicyTemplateGrants, 2> {
        self.bind(client, &params.template_id, &params.template_version)
    }
}
pub struct ListPolicyTemplateAffectedTeamsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_policy_template_affected_teams() -> ListPolicyTemplateAffectedTeamsStmt {
    ListPolicyTemplateAffectedTeamsStmt(
        "SELECT team FROM policy_template_affected_teams WHERE template_id = $1::text AND template_version = $2 ORDER BY team",
        None,
    )
}
impl ListPolicyTemplateAffectedTeamsStmt {
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
        template_id: &'a T1,
        template_version: &'a i32,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [template_id, template_version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListPolicyTemplateAffectedTeamsParams<T1>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for ListPolicyTemplateAffectedTeamsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListPolicyTemplateAffectedTeamsParams<T1>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.template_id, &params.template_version)
    }
}
pub struct ListPolicyTemplateApprovalTeamsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_policy_template_approval_teams() -> ListPolicyTemplateApprovalTeamsStmt {
    ListPolicyTemplateApprovalTeamsStmt(
        "SELECT team, required_approvals FROM policy_template_approval_teams WHERE template_id = $1::text AND template_version = $2 ORDER BY team",
        None,
    )
}
impl ListPolicyTemplateApprovalTeamsStmt {
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
        template_id: &'a T1,
        template_version: &'a i32,
    ) -> ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C, ListPolicyTemplateApprovalTeams, 2>
    {
        ListPolicyTemplateApprovalTeamsQuery {
            client,
            params: [template_id, template_version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                ListPolicyTemplateApprovalTeamsBorrowed,
                tokio_postgres::Error,
            > {
                Ok(ListPolicyTemplateApprovalTeamsBorrowed {
                    team: row.try_get(0)?,
                    required_approvals: row.try_get(1)?,
                })
            },
            mapper: |it| ListPolicyTemplateApprovalTeams::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListPolicyTemplateApprovalTeamsParams<T1>,
        ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C, ListPolicyTemplateApprovalTeams, 2>,
        C,
    > for ListPolicyTemplateApprovalTeamsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListPolicyTemplateApprovalTeamsParams<T1>,
    ) -> ListPolicyTemplateApprovalTeamsQuery<'c, 'a, 's, C, ListPolicyTemplateApprovalTeams, 2>
    {
        self.bind(client, &params.template_id, &params.template_version)
    }
}
pub struct ListPolicyTemplateGuardrailsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_policy_template_guardrails() -> ListPolicyTemplateGuardrailsStmt {
    ListPolicyTemplateGuardrailsStmt(
        "SELECT guardrail_key, name, COALESCE(description, '') AS description, enforcement, configuration FROM policy_template_guardrails WHERE template_id = $1::text AND template_version = $2 ORDER BY guardrail_key",
        None,
    )
}
impl ListPolicyTemplateGuardrailsStmt {
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
        template_id: &'a T1,
        template_version: &'a i32,
    ) -> ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C, ListPolicyTemplateGuardrails, 2> {
        ListPolicyTemplateGuardrailsQuery {
            client,
            params: [template_id, template_version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListPolicyTemplateGuardrailsBorrowed, tokio_postgres::Error> {
                Ok(ListPolicyTemplateGuardrailsBorrowed {
                    guardrail_key: row.try_get(0)?,
                    name: row.try_get(1)?,
                    description: row.try_get(2)?,
                    enforcement: row.try_get(3)?,
                    configuration: row.try_get(4)?,
                })
            },
            mapper: |it| ListPolicyTemplateGuardrails::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListPolicyTemplateGuardrailsParams<T1>,
        ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C, ListPolicyTemplateGuardrails, 2>,
        C,
    > for ListPolicyTemplateGuardrailsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListPolicyTemplateGuardrailsParams<T1>,
    ) -> ListPolicyTemplateGuardrailsQuery<'c, 'a, 's, C, ListPolicyTemplateGuardrails, 2> {
        self.bind(client, &params.template_id, &params.template_version)
    }
}
