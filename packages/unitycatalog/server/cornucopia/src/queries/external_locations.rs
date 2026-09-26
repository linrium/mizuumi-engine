// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateExternalLocationParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub name: T1,
    pub url: T2,
    pub comment: T3,
    pub credential_name: T4,
}
#[derive(Debug)]
pub struct ListExternalLocationsParams<T1: crate::StringSql> {
    pub page_token: T1,
    pub limit_value: i64,
}
#[derive(Debug)]
pub struct UpdateExternalLocationParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub new_name: T1,
    pub url: T2,
    pub comment: T3,
    pub name: T4,
    pub credential_name: T5,
}
#[derive(Debug)]
pub struct FindOverlappingExternalLocationParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub exclude_id: T1,
    pub url: T2,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateExternalLocation {
    pub name: String,
    pub id: String,
    pub url: String,
    pub credential_name: String,
    pub comment: String,
    pub owner: String,
    pub credential_id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
}
pub struct CreateExternalLocationBorrowed<'a> {
    pub name: &'a str,
    pub id: &'a str,
    pub url: &'a str,
    pub credential_name: &'a str,
    pub comment: &'a str,
    pub owner: &'a str,
    pub credential_id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
}
impl<'a> From<CreateExternalLocationBorrowed<'a>> for CreateExternalLocation {
    fn from(
        CreateExternalLocationBorrowed {
            name,
            id,
            url,
            credential_name,
            comment,
            owner,
            credential_id,
            created_at,
            created_by,
            updated_at,
            updated_by,
        }: CreateExternalLocationBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            comment: comment.into(),
            owner: owner.into(),
            credential_id: credential_id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListExternalLocations {
    pub name: String,
    pub id: String,
    pub url: String,
    pub credential_name: String,
    pub comment: String,
    pub owner: String,
    pub credential_id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
}
pub struct ListExternalLocationsBorrowed<'a> {
    pub name: &'a str,
    pub id: &'a str,
    pub url: &'a str,
    pub credential_name: &'a str,
    pub comment: &'a str,
    pub owner: &'a str,
    pub credential_id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
}
impl<'a> From<ListExternalLocationsBorrowed<'a>> for ListExternalLocations {
    fn from(
        ListExternalLocationsBorrowed {
            name,
            id,
            url,
            credential_name,
            comment,
            owner,
            credential_id,
            created_at,
            created_by,
            updated_at,
            updated_by,
        }: ListExternalLocationsBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            comment: comment.into(),
            owner: owner.into(),
            credential_id: credential_id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetExternalLocation {
    pub name: String,
    pub id: String,
    pub url: String,
    pub credential_name: String,
    pub comment: String,
    pub owner: String,
    pub credential_id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
}
pub struct GetExternalLocationBorrowed<'a> {
    pub name: &'a str,
    pub id: &'a str,
    pub url: &'a str,
    pub credential_name: &'a str,
    pub comment: &'a str,
    pub owner: &'a str,
    pub credential_id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
}
impl<'a> From<GetExternalLocationBorrowed<'a>> for GetExternalLocation {
    fn from(
        GetExternalLocationBorrowed {
            name,
            id,
            url,
            credential_name,
            comment,
            owner,
            credential_id,
            created_at,
            created_by,
            updated_at,
            updated_by,
        }: GetExternalLocationBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            comment: comment.into(),
            owner: owner.into(),
            credential_id: credential_id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateExternalLocation {
    pub name: String,
    pub id: String,
    pub url: String,
    pub credential_name: String,
    pub comment: String,
    pub owner: String,
    pub credential_id: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
}
pub struct UpdateExternalLocationBorrowed<'a> {
    pub name: &'a str,
    pub id: &'a str,
    pub url: &'a str,
    pub credential_name: &'a str,
    pub comment: &'a str,
    pub owner: &'a str,
    pub credential_id: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
}
impl<'a> From<UpdateExternalLocationBorrowed<'a>> for UpdateExternalLocation {
    fn from(
        UpdateExternalLocationBorrowed {
            name,
            id,
            url,
            credential_name,
            comment,
            owner,
            credential_id,
            created_at,
            created_by,
            updated_at,
            updated_by,
        }: UpdateExternalLocationBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            comment: comment.into(),
            owner: owner.into(),
            credential_id: credential_id.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CreateExternalLocationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CreateExternalLocationBorrowed, tokio_postgres::Error>,
    mapper: fn(CreateExternalLocationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateExternalLocationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateExternalLocationBorrowed) -> R,
    ) -> CreateExternalLocationQuery<'c, 'a, 's, C, R, N> {
        CreateExternalLocationQuery {
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
pub struct ListExternalLocationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListExternalLocationsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListExternalLocationsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListExternalLocationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListExternalLocationsBorrowed) -> R,
    ) -> ListExternalLocationsQuery<'c, 'a, 's, C, R, N> {
        ListExternalLocationsQuery {
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
pub struct GetExternalLocationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<GetExternalLocationBorrowed, tokio_postgres::Error>,
    mapper: fn(GetExternalLocationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetExternalLocationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetExternalLocationBorrowed) -> R,
    ) -> GetExternalLocationQuery<'c, 'a, 's, C, R, N> {
        GetExternalLocationQuery {
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
pub struct UpdateExternalLocationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<UpdateExternalLocationBorrowed, tokio_postgres::Error>,
    mapper: fn(UpdateExternalLocationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UpdateExternalLocationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UpdateExternalLocationBorrowed) -> R,
    ) -> UpdateExternalLocationQuery<'c, 'a, 's, C, R, N> {
        UpdateExternalLocationQuery {
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
pub struct CreateExternalLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_external_location() -> CreateExternalLocationStmt {
    CreateExternalLocationStmt(
        "WITH next_location AS ( SELECT gen_random_uuid()::text AS id, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms ), inserted AS ( INSERT INTO uc_external_locations ( id, name, url, credential_id, comment, owner, created_at, created_by, updated_at, updated_by ) SELECT next_location.id, $1::text, $2::text, credentials.id, $3::text, 'system', next_location.now_ms, 'system', next_location.now_ms, 'system' FROM next_location JOIN uc_credentials credentials ON credentials.name = $4::text RETURNING * ) SELECT inserted.name, inserted.id, inserted.url, credentials.name AS credential_name, COALESCE(inserted.comment, '') AS comment, COALESCE(inserted.owner, '') AS owner, inserted.credential_id, inserted.created_at, COALESCE(inserted.created_by, '') AS created_by, inserted.updated_at, COALESCE(inserted.updated_by, '') AS updated_by FROM inserted JOIN uc_credentials credentials ON credentials.id = inserted.credential_id",
        None,
    )
}
impl CreateExternalLocationStmt {
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
        url: &'a T2,
        comment: &'a T3,
        credential_name: &'a T4,
    ) -> CreateExternalLocationQuery<'c, 'a, 's, C, CreateExternalLocation, 4> {
        CreateExternalLocationQuery {
            client,
            params: [name, url, comment, credential_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CreateExternalLocationBorrowed, tokio_postgres::Error> {
                Ok(CreateExternalLocationBorrowed {
                    name: row.try_get(0)?,
                    id: row.try_get(1)?,
                    url: row.try_get(2)?,
                    credential_name: row.try_get(3)?,
                    comment: row.try_get(4)?,
                    owner: row.try_get(5)?,
                    credential_id: row.try_get(6)?,
                    created_at: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                    updated_by: row.try_get(10)?,
                })
            },
            mapper: |it| CreateExternalLocation::from(it),
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
        CreateExternalLocationParams<T1, T2, T3, T4>,
        CreateExternalLocationQuery<'c, 'a, 's, C, CreateExternalLocation, 4>,
        C,
    > for CreateExternalLocationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateExternalLocationParams<T1, T2, T3, T4>,
    ) -> CreateExternalLocationQuery<'c, 'a, 's, C, CreateExternalLocation, 4> {
        self.bind(
            client,
            &params.name,
            &params.url,
            &params.comment,
            &params.credential_name,
        )
    }
}
pub struct ListExternalLocationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_external_locations() -> ListExternalLocationsStmt {
    ListExternalLocationsStmt(
        "SELECT locations.name, locations.id, locations.url, COALESCE(credentials.name, '') AS credential_name, COALESCE(locations.comment, '') AS comment, COALESCE(locations.owner, '') AS owner, locations.credential_id, locations.created_at, COALESCE(locations.created_by, '') AS created_by, locations.updated_at, COALESCE(locations.updated_by, '') AS updated_by FROM uc_external_locations locations LEFT JOIN uc_credentials credentials ON credentials.id = locations.credential_id WHERE ($1::text = '' OR locations.name > $1::text) ORDER BY locations.name LIMIT $2",
        None,
    )
}
impl ListExternalLocationsStmt {
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
        page_token: &'a T1,
        limit_value: &'a i64,
    ) -> ListExternalLocationsQuery<'c, 'a, 's, C, ListExternalLocations, 2> {
        ListExternalLocationsQuery {
            client,
            params: [page_token, limit_value],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListExternalLocationsBorrowed, tokio_postgres::Error> {
                Ok(ListExternalLocationsBorrowed {
                    name: row.try_get(0)?,
                    id: row.try_get(1)?,
                    url: row.try_get(2)?,
                    credential_name: row.try_get(3)?,
                    comment: row.try_get(4)?,
                    owner: row.try_get(5)?,
                    credential_id: row.try_get(6)?,
                    created_at: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                    updated_by: row.try_get(10)?,
                })
            },
            mapper: |it| ListExternalLocations::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListExternalLocationsParams<T1>,
        ListExternalLocationsQuery<'c, 'a, 's, C, ListExternalLocations, 2>,
        C,
    > for ListExternalLocationsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListExternalLocationsParams<T1>,
    ) -> ListExternalLocationsQuery<'c, 'a, 's, C, ListExternalLocations, 2> {
        self.bind(client, &params.page_token, &params.limit_value)
    }
}
pub struct GetExternalLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_external_location() -> GetExternalLocationStmt {
    GetExternalLocationStmt(
        "SELECT locations.name, locations.id, locations.url, COALESCE(credentials.name, '') AS credential_name, COALESCE(locations.comment, '') AS comment, COALESCE(locations.owner, '') AS owner, locations.credential_id, locations.created_at, COALESCE(locations.created_by, '') AS created_by, locations.updated_at, COALESCE(locations.updated_by, '') AS updated_by FROM uc_external_locations locations LEFT JOIN uc_credentials credentials ON credentials.id = locations.credential_id WHERE locations.name = $1::text",
        None,
    )
}
impl GetExternalLocationStmt {
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
    ) -> GetExternalLocationQuery<'c, 'a, 's, C, GetExternalLocation, 1> {
        GetExternalLocationQuery {
            client,
            params: [name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetExternalLocationBorrowed, tokio_postgres::Error> {
                Ok(GetExternalLocationBorrowed {
                    name: row.try_get(0)?,
                    id: row.try_get(1)?,
                    url: row.try_get(2)?,
                    credential_name: row.try_get(3)?,
                    comment: row.try_get(4)?,
                    owner: row.try_get(5)?,
                    credential_id: row.try_get(6)?,
                    created_at: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                    updated_by: row.try_get(10)?,
                })
            },
            mapper: |it| GetExternalLocation::from(it),
        }
    }
}
pub struct UpdateExternalLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_external_location() -> UpdateExternalLocationStmt {
    UpdateExternalLocationStmt(
        "WITH updated AS ( UPDATE uc_external_locations locations SET name = $1::text, url = $2::text, credential_id = credentials.id, comment = $3::text, updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint, updated_by = 'system' FROM uc_credentials credentials WHERE locations.name = $4::text AND credentials.name = $5::text RETURNING locations.* ) SELECT updated.name, updated.id, updated.url, credentials.name AS credential_name, COALESCE(updated.comment, '') AS comment, COALESCE(updated.owner, '') AS owner, updated.credential_id, updated.created_at, COALESCE(updated.created_by, '') AS created_by, updated.updated_at, COALESCE(updated.updated_by, '') AS updated_by FROM updated JOIN uc_credentials credentials ON credentials.id = updated.credential_id",
        None,
    )
}
impl UpdateExternalLocationStmt {
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
        new_name: &'a T1,
        url: &'a T2,
        comment: &'a T3,
        name: &'a T4,
        credential_name: &'a T5,
    ) -> UpdateExternalLocationQuery<'c, 'a, 's, C, UpdateExternalLocation, 5> {
        UpdateExternalLocationQuery {
            client,
            params: [new_name, url, comment, name, credential_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<UpdateExternalLocationBorrowed, tokio_postgres::Error> {
                Ok(UpdateExternalLocationBorrowed {
                    name: row.try_get(0)?,
                    id: row.try_get(1)?,
                    url: row.try_get(2)?,
                    credential_name: row.try_get(3)?,
                    comment: row.try_get(4)?,
                    owner: row.try_get(5)?,
                    credential_id: row.try_get(6)?,
                    created_at: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                    updated_by: row.try_get(10)?,
                })
            },
            mapper: |it| UpdateExternalLocation::from(it),
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
        UpdateExternalLocationParams<T1, T2, T3, T4, T5>,
        UpdateExternalLocationQuery<'c, 'a, 's, C, UpdateExternalLocation, 5>,
        C,
    > for UpdateExternalLocationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpdateExternalLocationParams<T1, T2, T3, T4, T5>,
    ) -> UpdateExternalLocationQuery<'c, 'a, 's, C, UpdateExternalLocation, 5> {
        self.bind(
            client,
            &params.new_name,
            &params.url,
            &params.comment,
            &params.name,
            &params.credential_name,
        )
    }
}
pub struct FindOverlappingExternalLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_overlapping_external_location() -> FindOverlappingExternalLocationStmt {
    FindOverlappingExternalLocationStmt(
        "SELECT name FROM uc_external_locations WHERE ($1::text = '' OR id <> $1::text) AND ( url = $2::text OR starts_with($2::text, url || CASE WHEN right(url, 1) = '/' THEN '' ELSE '/' END) OR starts_with(url, $2::text || CASE WHEN right($2::text, 1) = '/' THEN '' ELSE '/' END) ) LIMIT 1",
        None,
    )
}
impl FindOverlappingExternalLocationStmt {
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
        exclude_id: &'a T1,
        url: &'a T2,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [exclude_id, url],
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
        FindOverlappingExternalLocationParams<T1, T2>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for FindOverlappingExternalLocationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a FindOverlappingExternalLocationParams<T1, T2>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.exclude_id, &params.url)
    }
}
pub struct FindExternalTableUsingLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_external_table_using_location() -> FindExternalTableUsingLocationStmt {
    FindExternalTableUsingLocationStmt(
        "SELECT tables.id FROM uc_tables tables WHERE tables.storage_location = $1::text OR starts_with( tables.storage_location, $1::text || CASE WHEN right($1::text, 1) = '/' THEN '' ELSE '/' END ) LIMIT 1",
        None,
    )
}
impl FindExternalTableUsingLocationStmt {
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
        url: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [url],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct DeleteExternalLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_external_location() -> DeleteExternalLocationStmt {
    DeleteExternalLocationStmt(
        "WITH deleted AS ( DELETE FROM uc_external_locations WHERE name = $1::text RETURNING id ), deleted_permissions AS ( DELETE FROM uc_permissions WHERE resource_id IN (SELECT id FROM deleted) ) SELECT id FROM deleted",
        None,
    )
}
impl DeleteExternalLocationStmt {
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
pub struct FindExternalLocationUsingCredentialStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_external_location_using_credential() -> FindExternalLocationUsingCredentialStmt {
    FindExternalLocationUsingCredentialStmt(
        "SELECT locations.name FROM uc_external_locations locations JOIN uc_credentials credentials ON credentials.id = locations.credential_id WHERE credentials.name = $1::text LIMIT 1",
        None,
    )
}
impl FindExternalLocationUsingCredentialStmt {
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
        credential_name: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [credential_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct FindExternalLocationForPathStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_external_location_for_path() -> FindExternalLocationForPathStmt {
    FindExternalLocationForPathStmt(
        "SELECT url FROM uc_external_locations WHERE url = $1::text OR starts_with( $1::text, url || CASE WHEN right(url, 1) = '/' THEN '' ELSE '/' END ) ORDER BY length(url) DESC LIMIT 1",
        None,
    )
}
impl FindExternalLocationForPathStmt {
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
        url: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [url],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
