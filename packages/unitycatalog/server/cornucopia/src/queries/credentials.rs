// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateCredentialParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub name: T1,
    pub role_arn: T2,
    pub purpose: T3,
    pub comment: T4,
}
#[derive(Debug)]
pub struct ListCredentialsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub page_token: T1,
    pub purpose: T2,
    pub limit_value: i64,
}
#[derive(Debug)]
pub struct UpdateCredentialParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub new_name: T1,
    pub role_arn: T2,
    pub comment: T3,
    pub name: T4,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateCredential {
    pub name: String,
    pub credential: serde_json::Value,
    pub comment: String,
    pub owner: String,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub purpose: String,
}
pub struct CreateCredentialBorrowed<'a> {
    pub name: &'a str,
    pub credential: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub comment: &'a str,
    pub owner: &'a str,
    pub full_name: &'a str,
    pub id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub purpose: &'a str,
}
impl<'a> From<CreateCredentialBorrowed<'a>> for CreateCredential {
    fn from(
        CreateCredentialBorrowed {
            name,
            credential,
            comment,
            owner,
            full_name,
            id,
            created_at,
            created_by,
            updated_at,
            updated_by,
            purpose,
        }: CreateCredentialBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            credential: serde_json::from_str(credential.0.get()).unwrap(),
            comment: comment.into(),
            owner: owner.into(),
            full_name: full_name.into(),
            id: id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            purpose: purpose.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListCredentials {
    pub name: String,
    pub credential: serde_json::Value,
    pub comment: String,
    pub owner: String,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub purpose: String,
}
pub struct ListCredentialsBorrowed<'a> {
    pub name: &'a str,
    pub credential: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub comment: &'a str,
    pub owner: &'a str,
    pub full_name: &'a str,
    pub id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub purpose: &'a str,
}
impl<'a> From<ListCredentialsBorrowed<'a>> for ListCredentials {
    fn from(
        ListCredentialsBorrowed {
            name,
            credential,
            comment,
            owner,
            full_name,
            id,
            created_at,
            created_by,
            updated_at,
            updated_by,
            purpose,
        }: ListCredentialsBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            credential: serde_json::from_str(credential.0.get()).unwrap(),
            comment: comment.into(),
            owner: owner.into(),
            full_name: full_name.into(),
            id: id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            purpose: purpose.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetCredential {
    pub name: String,
    pub credential: serde_json::Value,
    pub comment: String,
    pub owner: String,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub purpose: String,
}
pub struct GetCredentialBorrowed<'a> {
    pub name: &'a str,
    pub credential: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub comment: &'a str,
    pub owner: &'a str,
    pub full_name: &'a str,
    pub id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub purpose: &'a str,
}
impl<'a> From<GetCredentialBorrowed<'a>> for GetCredential {
    fn from(
        GetCredentialBorrowed {
            name,
            credential,
            comment,
            owner,
            full_name,
            id,
            created_at,
            created_by,
            updated_at,
            updated_by,
            purpose,
        }: GetCredentialBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            credential: serde_json::from_str(credential.0.get()).unwrap(),
            comment: comment.into(),
            owner: owner.into(),
            full_name: full_name.into(),
            id: id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            purpose: purpose.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateCredential {
    pub name: String,
    pub credential: serde_json::Value,
    pub comment: String,
    pub owner: String,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub purpose: String,
}
pub struct UpdateCredentialBorrowed<'a> {
    pub name: &'a str,
    pub credential: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub comment: &'a str,
    pub owner: &'a str,
    pub full_name: &'a str,
    pub id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub purpose: &'a str,
}
impl<'a> From<UpdateCredentialBorrowed<'a>> for UpdateCredential {
    fn from(
        UpdateCredentialBorrowed {
            name,
            credential,
            comment,
            owner,
            full_name,
            id,
            created_at,
            created_by,
            updated_at,
            updated_by,
            purpose,
        }: UpdateCredentialBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            credential: serde_json::from_str(credential.0.get()).unwrap(),
            comment: comment.into(),
            owner: owner.into(),
            full_name: full_name.into(),
            id: id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            purpose: purpose.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CreateCredentialQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CreateCredentialBorrowed, tokio_postgres::Error>,
    mapper: fn(CreateCredentialBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateCredentialQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateCredentialBorrowed) -> R,
    ) -> CreateCredentialQuery<'c, 'a, 's, C, R, N> {
        CreateCredentialQuery {
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
pub struct ListCredentialsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListCredentialsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListCredentialsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListCredentialsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListCredentialsBorrowed) -> R,
    ) -> ListCredentialsQuery<'c, 'a, 's, C, R, N> {
        ListCredentialsQuery {
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
pub struct GetCredentialQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetCredentialBorrowed, tokio_postgres::Error>,
    mapper: fn(GetCredentialBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetCredentialQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetCredentialBorrowed) -> R,
    ) -> GetCredentialQuery<'c, 'a, 's, C, R, N> {
        GetCredentialQuery {
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
pub struct UpdateCredentialQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UpdateCredentialBorrowed, tokio_postgres::Error>,
    mapper: fn(UpdateCredentialBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UpdateCredentialQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UpdateCredentialBorrowed) -> R,
    ) -> UpdateCredentialQuery<'c, 'a, 's, C, R, N> {
        UpdateCredentialQuery {
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
pub struct CreateCredentialStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_credential() -> CreateCredentialStmt {
    CreateCredentialStmt(
        "WITH next_credential AS ( SELECT gen_random_uuid()::text AS id, gen_random_uuid()::text AS external_id, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms ), inserted AS ( INSERT INTO uc_credentials ( id, name, credential_type, credential, purpose, comment, owner, created_at, created_by, updated_at, updated_by ) SELECT next_credential.id, $1::text, 'AWS_IAM_ROLE', jsonb_build_object('role_arn', $2::text, 'external_id', next_credential.external_id), $3::text, $4::text, 'system', next_credential.now_ms, 'system', next_credential.now_ms, 'system' FROM next_credential RETURNING * ) SELECT name, credential, COALESCE(comment, '') AS comment, COALESCE(owner, '') AS owner, name AS full_name, id, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, purpose FROM inserted",
        None,
    )
}
impl CreateCredentialStmt {
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
        name: &'a T1,
        role_arn: &'a T2,
        purpose: &'a T3,
        comment: &'a T4,
    ) -> CreateCredentialQuery<'c, 'a, 's, C, CreateCredential, 4> {
        CreateCredentialQuery {
            client,
            params: [name, role_arn, purpose, comment],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CreateCredentialBorrowed, tokio_postgres::Error> {
                Ok(CreateCredentialBorrowed {
                    name: row.try_get(0)?,
                    credential: row.try_get(1)?,
                    comment: row.try_get(2)?,
                    owner: row.try_get(3)?,
                    full_name: row.try_get(4)?,
                    id: row.try_get(5)?,
                    created_at: row.try_get(6)?,
                    created_by: row.try_get(7)?,
                    updated_at: row.try_get(8)?,
                    updated_by: row.try_get(9)?,
                    purpose: row.try_get(10)?,
                })
            },
            mapper: |it| CreateCredential::from(it),
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
        CreateCredentialParams<T1, T2, T3, T4>,
        CreateCredentialQuery<'c, 'a, 's, C, CreateCredential, 4>,
        C,
    > for CreateCredentialStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateCredentialParams<T1, T2, T3, T4>,
    ) -> CreateCredentialQuery<'c, 'a, 's, C, CreateCredential, 4> {
        self.bind(
            client,
            &params.name,
            &params.role_arn,
            &params.purpose,
            &params.comment,
        )
    }
}
pub struct ListCredentialsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_credentials() -> ListCredentialsStmt {
    ListCredentialsStmt(
        "SELECT name, credential, COALESCE(comment, '') AS comment, COALESCE(owner, '') AS owner, name AS full_name, id, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, purpose FROM uc_credentials WHERE ($1::text = '' OR name > $1::text) AND ($2::text = '' OR purpose = $2::text) ORDER BY name LIMIT $3",
        None,
    )
}
impl ListCredentialsStmt {
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
        page_token: &'a T1,
        purpose: &'a T2,
        limit_value: &'a i64,
    ) -> ListCredentialsQuery<'c, 'a, 's, C, ListCredentials, 3> {
        ListCredentialsQuery {
            client,
            params: [page_token, purpose, limit_value],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListCredentialsBorrowed, tokio_postgres::Error> {
                Ok(ListCredentialsBorrowed {
                    name: row.try_get(0)?,
                    credential: row.try_get(1)?,
                    comment: row.try_get(2)?,
                    owner: row.try_get(3)?,
                    full_name: row.try_get(4)?,
                    id: row.try_get(5)?,
                    created_at: row.try_get(6)?,
                    created_by: row.try_get(7)?,
                    updated_at: row.try_get(8)?,
                    updated_by: row.try_get(9)?,
                    purpose: row.try_get(10)?,
                })
            },
            mapper: |it| ListCredentials::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListCredentialsParams<T1, T2>,
        ListCredentialsQuery<'c, 'a, 's, C, ListCredentials, 3>,
        C,
    > for ListCredentialsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListCredentialsParams<T1, T2>,
    ) -> ListCredentialsQuery<'c, 'a, 's, C, ListCredentials, 3> {
        self.bind(
            client,
            &params.page_token,
            &params.purpose,
            &params.limit_value,
        )
    }
}
pub struct GetCredentialStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_credential() -> GetCredentialStmt {
    GetCredentialStmt(
        "SELECT name, credential, COALESCE(comment, '') AS comment, COALESCE(owner, '') AS owner, name AS full_name, id, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, purpose FROM uc_credentials WHERE name = $1::text",
        None,
    )
}
impl GetCredentialStmt {
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
        name: &'a T1,
    ) -> GetCredentialQuery<'c, 'a, 's, C, GetCredential, 1> {
        GetCredentialQuery {
            client,
            params: [name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<GetCredentialBorrowed, tokio_postgres::Error> {
                    Ok(GetCredentialBorrowed {
                        name: row.try_get(0)?,
                        credential: row.try_get(1)?,
                        comment: row.try_get(2)?,
                        owner: row.try_get(3)?,
                        full_name: row.try_get(4)?,
                        id: row.try_get(5)?,
                        created_at: row.try_get(6)?,
                        created_by: row.try_get(7)?,
                        updated_at: row.try_get(8)?,
                        updated_by: row.try_get(9)?,
                        purpose: row.try_get(10)?,
                    })
                },
            mapper: |it| GetCredential::from(it),
        }
    }
}
pub struct UpdateCredentialStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_credential() -> UpdateCredentialStmt {
    UpdateCredentialStmt(
        "UPDATE uc_credentials SET name = COALESCE($1::text, name), credential = CASE WHEN $2::text = '' THEN credential ELSE jsonb_build_object('role_arn', $2::text, 'external_id', gen_random_uuid()::text) END, comment = COALESCE($3::text, comment), updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint, updated_by = 'system' WHERE name = $4::text RETURNING name, credential, COALESCE(comment, '') AS comment, COALESCE(owner, '') AS owner, name AS full_name, id, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, purpose",
        None,
    )
}
impl UpdateCredentialStmt {
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
        new_name: &'a T1,
        role_arn: &'a T2,
        comment: &'a T3,
        name: &'a T4,
    ) -> UpdateCredentialQuery<'c, 'a, 's, C, UpdateCredential, 4> {
        UpdateCredentialQuery {
            client,
            params: [new_name, role_arn, comment, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<UpdateCredentialBorrowed, tokio_postgres::Error> {
                Ok(UpdateCredentialBorrowed {
                    name: row.try_get(0)?,
                    credential: row.try_get(1)?,
                    comment: row.try_get(2)?,
                    owner: row.try_get(3)?,
                    full_name: row.try_get(4)?,
                    id: row.try_get(5)?,
                    created_at: row.try_get(6)?,
                    created_by: row.try_get(7)?,
                    updated_at: row.try_get(8)?,
                    updated_by: row.try_get(9)?,
                    purpose: row.try_get(10)?,
                })
            },
            mapper: |it| UpdateCredential::from(it),
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
        UpdateCredentialParams<T1, T2, T3, T4>,
        UpdateCredentialQuery<'c, 'a, 's, C, UpdateCredential, 4>,
        C,
    > for UpdateCredentialStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpdateCredentialParams<T1, T2, T3, T4>,
    ) -> UpdateCredentialQuery<'c, 'a, 's, C, UpdateCredential, 4> {
        self.bind(
            client,
            &params.new_name,
            &params.role_arn,
            &params.comment,
            &params.name,
        )
    }
}
pub struct DeleteCredentialStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_credential() -> DeleteCredentialStmt {
    DeleteCredentialStmt(
        "DELETE FROM uc_credentials WHERE name = $1::text RETURNING id",
        None,
    )
}
impl DeleteCredentialStmt {
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
        name: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
