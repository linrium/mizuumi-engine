// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateCatalogParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::JsonSql,
    T4: crate::StringSql,
> {
    pub name: T1,
    pub comment: T2,
    pub properties: T3,
    pub storage_root: T4,
}
#[derive(Debug)]
pub struct ListCatalogsParams<T1: crate::StringSql> {
    pub page_token: T1,
    pub limit_value: i64,
}
#[derive(Debug)]
pub struct UpdateCatalogParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::JsonSql,
    T4: crate::StringSql,
> {
    pub new_name: T1,
    pub comment: T2,
    pub properties: T3,
    pub name: T4,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateCatalog {
    pub name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct CreateCatalogBorrowed<'a> {
    pub name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<CreateCatalogBorrowed<'a>> for CreateCatalog {
    fn from(
        CreateCatalogBorrowed {
            name,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            id,
            storage_root,
            storage_location,
        }: CreateCatalogBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            id: id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListCatalogs {
    pub name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct ListCatalogsBorrowed<'a> {
    pub name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<ListCatalogsBorrowed<'a>> for ListCatalogs {
    fn from(
        ListCatalogsBorrowed {
            name,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            id,
            storage_root,
            storage_location,
        }: ListCatalogsBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            id: id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetCatalog {
    pub name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct GetCatalogBorrowed<'a> {
    pub name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<GetCatalogBorrowed<'a>> for GetCatalog {
    fn from(
        GetCatalogBorrowed {
            name,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            id,
            storage_root,
            storage_location,
        }: GetCatalogBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            id: id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateCatalog {
    pub name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct UpdateCatalogBorrowed<'a> {
    pub name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<UpdateCatalogBorrowed<'a>> for UpdateCatalog {
    fn from(
        UpdateCatalogBorrowed {
            name,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            id,
            storage_root,
            storage_location,
        }: UpdateCatalogBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            id: id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CreateCatalogQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CreateCatalogBorrowed, tokio_postgres::Error>,
    mapper: fn(CreateCatalogBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateCatalogQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateCatalogBorrowed) -> R,
    ) -> CreateCatalogQuery<'c, 'a, 's, C, R, N> {
        CreateCatalogQuery {
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
pub struct ListCatalogsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListCatalogsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListCatalogsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListCatalogsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListCatalogsBorrowed) -> R,
    ) -> ListCatalogsQuery<'c, 'a, 's, C, R, N> {
        ListCatalogsQuery {
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
pub struct GetCatalogQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetCatalogBorrowed, tokio_postgres::Error>,
    mapper: fn(GetCatalogBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetCatalogQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetCatalogBorrowed) -> R,
    ) -> GetCatalogQuery<'c, 'a, 's, C, R, N> {
        GetCatalogQuery {
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
pub struct UpdateCatalogQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UpdateCatalogBorrowed, tokio_postgres::Error>,
    mapper: fn(UpdateCatalogBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UpdateCatalogQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UpdateCatalogBorrowed) -> R,
    ) -> UpdateCatalogQuery<'c, 'a, 's, C, R, N> {
        UpdateCatalogQuery {
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
pub struct CreateCatalogStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_catalog() -> CreateCatalogStmt {
    CreateCatalogStmt(
        "WITH next_catalog AS ( SELECT gen_random_uuid()::text AS id, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms ) INSERT INTO uc_catalogs ( id, name, comment, properties, owner, created_at, created_by, updated_at, updated_by, storage_root, storage_location ) SELECT next_catalog.id, $1, $2, COALESCE($3, '{}'::jsonb), 'system', next_catalog.now_ms, 'system', next_catalog.now_ms, 'system', NULLIF($4, ''), CASE WHEN $4 = '' THEN NULL ELSE CONCAT($4, '/__unitystorage/catalogs/', next_catalog.id) END FROM next_catalog RETURNING name, COALESCE(comment, '') AS comment, properties, COALESCE(owner, '') AS owner, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, id, COALESCE(storage_root, '') AS storage_root, COALESCE(storage_location, '') AS storage_location",
        None,
    )
}
impl CreateCatalogStmt {
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
        T3: crate::JsonSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        name: &'a T1,
        comment: &'a T2,
        properties: &'a T3,
        storage_root: &'a T4,
    ) -> CreateCatalogQuery<'c, 'a, 's, C, CreateCatalog, 4> {
        CreateCatalogQuery {
            client,
            params: [name, comment, properties, storage_root],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CreateCatalogBorrowed, tokio_postgres::Error> {
                    Ok(CreateCatalogBorrowed {
                        name: row.try_get(0)?,
                        comment: row.try_get(1)?,
                        properties: row.try_get(2)?,
                        owner: row.try_get(3)?,
                        created_at: row.try_get(4)?,
                        created_by: row.try_get(5)?,
                        updated_at: row.try_get(6)?,
                        updated_by: row.try_get(7)?,
                        id: row.try_get(8)?,
                        storage_root: row.try_get(9)?,
                        storage_location: row.try_get(10)?,
                    })
                },
            mapper: |it| CreateCatalog::from(it),
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
    T3: crate::JsonSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateCatalogParams<T1, T2, T3, T4>,
        CreateCatalogQuery<'c, 'a, 's, C, CreateCatalog, 4>,
        C,
    > for CreateCatalogStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateCatalogParams<T1, T2, T3, T4>,
    ) -> CreateCatalogQuery<'c, 'a, 's, C, CreateCatalog, 4> {
        self.bind(
            client,
            &params.name,
            &params.comment,
            &params.properties,
            &params.storage_root,
        )
    }
}
pub struct ListCatalogsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_catalogs() -> ListCatalogsStmt {
    ListCatalogsStmt(
        "SELECT name, COALESCE(comment, '') AS comment, properties, COALESCE(owner, '') AS owner, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, id, COALESCE(storage_root, '') AS storage_root, COALESCE(storage_location, '') AS storage_location FROM uc_catalogs WHERE ($1 = '' OR name > $1) ORDER BY name LIMIT $2",
        None,
    )
}
impl ListCatalogsStmt {
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
    ) -> ListCatalogsQuery<'c, 'a, 's, C, ListCatalogs, 2> {
        ListCatalogsQuery {
            client,
            params: [page_token, limit_value],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListCatalogsBorrowed, tokio_postgres::Error> {
                    Ok(ListCatalogsBorrowed {
                        name: row.try_get(0)?,
                        comment: row.try_get(1)?,
                        properties: row.try_get(2)?,
                        owner: row.try_get(3)?,
                        created_at: row.try_get(4)?,
                        created_by: row.try_get(5)?,
                        updated_at: row.try_get(6)?,
                        updated_by: row.try_get(7)?,
                        id: row.try_get(8)?,
                        storage_root: row.try_get(9)?,
                        storage_location: row.try_get(10)?,
                    })
                },
            mapper: |it| ListCatalogs::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListCatalogsParams<T1>,
        ListCatalogsQuery<'c, 'a, 's, C, ListCatalogs, 2>,
        C,
    > for ListCatalogsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListCatalogsParams<T1>,
    ) -> ListCatalogsQuery<'c, 'a, 's, C, ListCatalogs, 2> {
        self.bind(client, &params.page_token, &params.limit_value)
    }
}
pub struct GetCatalogStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_catalog() -> GetCatalogStmt {
    GetCatalogStmt(
        "SELECT name, COALESCE(comment, '') AS comment, properties, COALESCE(owner, '') AS owner, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, id, COALESCE(storage_root, '') AS storage_root, COALESCE(storage_location, '') AS storage_location FROM uc_catalogs WHERE name = $1",
        None,
    )
}
impl GetCatalogStmt {
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
    ) -> GetCatalogQuery<'c, 'a, 's, C, GetCatalog, 1> {
        GetCatalogQuery {
            client,
            params: [name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<GetCatalogBorrowed, tokio_postgres::Error> {
                    Ok(GetCatalogBorrowed {
                        name: row.try_get(0)?,
                        comment: row.try_get(1)?,
                        properties: row.try_get(2)?,
                        owner: row.try_get(3)?,
                        created_at: row.try_get(4)?,
                        created_by: row.try_get(5)?,
                        updated_at: row.try_get(6)?,
                        updated_by: row.try_get(7)?,
                        id: row.try_get(8)?,
                        storage_root: row.try_get(9)?,
                        storage_location: row.try_get(10)?,
                    })
                },
            mapper: |it| GetCatalog::from(it),
        }
    }
}
pub struct UpdateCatalogStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_catalog() -> UpdateCatalogStmt {
    UpdateCatalogStmt(
        "UPDATE uc_catalogs SET name = COALESCE($1, name), comment = COALESCE($2, comment), properties = COALESCE($3, properties), updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint, updated_by = 'system' WHERE name = $4 RETURNING name, COALESCE(comment, '') AS comment, properties, COALESCE(owner, '') AS owner, created_at, COALESCE(created_by, '') AS created_by, updated_at, COALESCE(updated_by, '') AS updated_by, id, COALESCE(storage_root, '') AS storage_root, COALESCE(storage_location, '') AS storage_location",
        None,
    )
}
impl UpdateCatalogStmt {
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
        T3: crate::JsonSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        new_name: &'a T1,
        comment: &'a T2,
        properties: &'a T3,
        name: &'a T4,
    ) -> UpdateCatalogQuery<'c, 'a, 's, C, UpdateCatalog, 4> {
        UpdateCatalogQuery {
            client,
            params: [new_name, comment, properties, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<UpdateCatalogBorrowed, tokio_postgres::Error> {
                    Ok(UpdateCatalogBorrowed {
                        name: row.try_get(0)?,
                        comment: row.try_get(1)?,
                        properties: row.try_get(2)?,
                        owner: row.try_get(3)?,
                        created_at: row.try_get(4)?,
                        created_by: row.try_get(5)?,
                        updated_at: row.try_get(6)?,
                        updated_by: row.try_get(7)?,
                        id: row.try_get(8)?,
                        storage_root: row.try_get(9)?,
                        storage_location: row.try_get(10)?,
                    })
                },
            mapper: |it| UpdateCatalog::from(it),
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
    T3: crate::JsonSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        UpdateCatalogParams<T1, T2, T3, T4>,
        UpdateCatalogQuery<'c, 'a, 's, C, UpdateCatalog, 4>,
        C,
    > for UpdateCatalogStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpdateCatalogParams<T1, T2, T3, T4>,
    ) -> UpdateCatalogQuery<'c, 'a, 's, C, UpdateCatalog, 4> {
        self.bind(
            client,
            &params.new_name,
            &params.comment,
            &params.properties,
            &params.name,
        )
    }
}
pub struct DeleteCatalogStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_catalog() -> DeleteCatalogStmt {
    DeleteCatalogStmt(
        "DELETE FROM uc_catalogs WHERE name = $1 RETURNING name",
        None,
    )
}
impl DeleteCatalogStmt {
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
