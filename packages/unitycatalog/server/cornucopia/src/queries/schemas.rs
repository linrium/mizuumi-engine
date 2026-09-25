// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateSchemaParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::JsonSql,
    T5: crate::StringSql,
> {
    pub catalog_name: T1,
    pub name: T2,
    pub comment: T3,
    pub properties: T4,
    pub storage_root: T5,
}
#[derive(Debug)]
pub struct ListSchemasParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub catalog_name: T1,
    pub page_token: T2,
    pub limit_value: i64,
}
#[derive(Debug)]
pub struct GetSchemaParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub catalog_name: T1,
    pub name: T2,
}
#[derive(Debug)]
pub struct UpdateSchemaParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::JsonSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub new_name: T1,
    pub comment: T2,
    pub properties: T3,
    pub catalog_name: T4,
    pub name: T5,
}
#[derive(Debug)]
pub struct DeleteSchemaParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub catalog_name: T1,
    pub name: T2,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateSchema {
    pub name: String,
    pub catalog_name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub full_name: String,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub schema_id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct CreateSchemaBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub full_name: &'a str,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub schema_id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<CreateSchemaBorrowed<'a>> for CreateSchema {
    fn from(
        CreateSchemaBorrowed {
            name,
            catalog_name,
            comment,
            properties,
            full_name,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            schema_id,
            storage_root,
            storage_location,
        }: CreateSchemaBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            full_name: full_name.into(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            schema_id: schema_id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListSchemas {
    pub name: String,
    pub catalog_name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub full_name: String,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub schema_id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct ListSchemasBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub full_name: &'a str,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub schema_id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<ListSchemasBorrowed<'a>> for ListSchemas {
    fn from(
        ListSchemasBorrowed {
            name,
            catalog_name,
            comment,
            properties,
            full_name,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            schema_id,
            storage_root,
            storage_location,
        }: ListSchemasBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            full_name: full_name.into(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            schema_id: schema_id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetSchema {
    pub name: String,
    pub catalog_name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub full_name: String,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub schema_id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct GetSchemaBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub full_name: &'a str,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub schema_id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<GetSchemaBorrowed<'a>> for GetSchema {
    fn from(
        GetSchemaBorrowed {
            name,
            catalog_name,
            comment,
            properties,
            full_name,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            schema_id,
            storage_root,
            storage_location,
        }: GetSchemaBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            full_name: full_name.into(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            schema_id: schema_id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateSchema {
    pub name: String,
    pub catalog_name: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub full_name: String,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub schema_id: String,
    pub storage_root: String,
    pub storage_location: String,
}
pub struct UpdateSchemaBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub full_name: &'a str,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub schema_id: &'a str,
    pub storage_root: &'a str,
    pub storage_location: &'a str,
}
impl<'a> From<UpdateSchemaBorrowed<'a>> for UpdateSchema {
    fn from(
        UpdateSchemaBorrowed {
            name,
            catalog_name,
            comment,
            properties,
            full_name,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            schema_id,
            storage_root,
            storage_location,
        }: UpdateSchemaBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            full_name: full_name.into(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            schema_id: schema_id.into(),
            storage_root: storage_root.into(),
            storage_location: storage_location.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CreateSchemaQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CreateSchemaBorrowed, tokio_postgres::Error>,
    mapper: fn(CreateSchemaBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateSchemaQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateSchemaBorrowed) -> R,
    ) -> CreateSchemaQuery<'c, 'a, 's, C, R, N> {
        CreateSchemaQuery {
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
pub struct ListSchemasQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListSchemasBorrowed, tokio_postgres::Error>,
    mapper: fn(ListSchemasBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListSchemasQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListSchemasBorrowed) -> R,
    ) -> ListSchemasQuery<'c, 'a, 's, C, R, N> {
        ListSchemasQuery {
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
pub struct GetSchemaQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetSchemaBorrowed, tokio_postgres::Error>,
    mapper: fn(GetSchemaBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetSchemaQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(GetSchemaBorrowed) -> R) -> GetSchemaQuery<'c, 'a, 's, C, R, N> {
        GetSchemaQuery {
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
pub struct UpdateSchemaQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UpdateSchemaBorrowed, tokio_postgres::Error>,
    mapper: fn(UpdateSchemaBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UpdateSchemaQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UpdateSchemaBorrowed) -> R,
    ) -> UpdateSchemaQuery<'c, 'a, 's, C, R, N> {
        UpdateSchemaQuery {
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
pub struct CreateSchemaStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_schema() -> CreateSchemaStmt {
    CreateSchemaStmt(
        "WITH catalog AS ( SELECT id, name FROM uc_catalogs WHERE name = $1 ), next_schema AS ( SELECT gen_random_uuid()::text AS id, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms ), inserted AS ( INSERT INTO uc_schemas ( id, catalog_id, name, comment, properties, owner, created_at, created_by, updated_at, updated_by, storage_root, storage_location ) SELECT next_schema.id, catalog.id, $2, $3, COALESCE($4, '{}'::jsonb), 'system', next_schema.now_ms, 'system', next_schema.now_ms, 'system', NULLIF($5, ''), CASE WHEN $5 = '' THEN NULL ELSE CONCAT($5, '/__unitystorage/schemas/', next_schema.id) END FROM catalog, next_schema RETURNING * ) SELECT inserted.name, catalog.name AS catalog_name, COALESCE(inserted.comment, '') AS comment, inserted.properties, CONCAT(catalog.name, '.', inserted.name) AS full_name, COALESCE(inserted.owner, '') AS owner, inserted.created_at, COALESCE(inserted.created_by, '') AS created_by, inserted.updated_at, COALESCE(inserted.updated_by, '') AS updated_by, inserted.id AS schema_id, COALESCE(inserted.storage_root, '') AS storage_root, COALESCE(inserted.storage_location, '') AS storage_location FROM inserted JOIN catalog ON catalog.id = inserted.catalog_id",
        None,
    )
}
impl CreateSchemaStmt {
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
        T4: crate::JsonSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        catalog_name: &'a T1,
        name: &'a T2,
        comment: &'a T3,
        properties: &'a T4,
        storage_root: &'a T5,
    ) -> CreateSchemaQuery<'c, 'a, 's, C, CreateSchema, 5> {
        CreateSchemaQuery {
            client,
            params: [catalog_name, name, comment, properties, storage_root],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CreateSchemaBorrowed, tokio_postgres::Error> {
                    Ok(CreateSchemaBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        comment: row.try_get(2)?,
                        properties: row.try_get(3)?,
                        full_name: row.try_get(4)?,
                        owner: row.try_get(5)?,
                        created_at: row.try_get(6)?,
                        created_by: row.try_get(7)?,
                        updated_at: row.try_get(8)?,
                        updated_by: row.try_get(9)?,
                        schema_id: row.try_get(10)?,
                        storage_root: row.try_get(11)?,
                        storage_location: row.try_get(12)?,
                    })
                },
            mapper: |it| CreateSchema::from(it),
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
    T4: crate::JsonSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateSchemaParams<T1, T2, T3, T4, T5>,
        CreateSchemaQuery<'c, 'a, 's, C, CreateSchema, 5>,
        C,
    > for CreateSchemaStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateSchemaParams<T1, T2, T3, T4, T5>,
    ) -> CreateSchemaQuery<'c, 'a, 's, C, CreateSchema, 5> {
        self.bind(
            client,
            &params.catalog_name,
            &params.name,
            &params.comment,
            &params.properties,
            &params.storage_root,
        )
    }
}
pub struct ListSchemasStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_schemas() -> ListSchemasStmt {
    ListSchemasStmt(
        "SELECT schemas.name, catalogs.name AS catalog_name, COALESCE(schemas.comment, '') AS comment, schemas.properties, CONCAT(catalogs.name, '.', schemas.name) AS full_name, COALESCE(schemas.owner, '') AS owner, schemas.created_at, COALESCE(schemas.created_by, '') AS created_by, schemas.updated_at, COALESCE(schemas.updated_by, '') AS updated_by, schemas.id AS schema_id, COALESCE(schemas.storage_root, '') AS storage_root, COALESCE(schemas.storage_location, '') AS storage_location FROM uc_schemas schemas JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1 AND ($2 = '' OR schemas.name > $2) ORDER BY schemas.name LIMIT $3",
        None,
    )
}
impl ListSchemasStmt {
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
        catalog_name: &'a T1,
        page_token: &'a T2,
        limit_value: &'a i64,
    ) -> ListSchemasQuery<'c, 'a, 's, C, ListSchemas, 3> {
        ListSchemasQuery {
            client,
            params: [catalog_name, page_token, limit_value],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListSchemasBorrowed, tokio_postgres::Error> {
                    Ok(ListSchemasBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        comment: row.try_get(2)?,
                        properties: row.try_get(3)?,
                        full_name: row.try_get(4)?,
                        owner: row.try_get(5)?,
                        created_at: row.try_get(6)?,
                        created_by: row.try_get(7)?,
                        updated_at: row.try_get(8)?,
                        updated_by: row.try_get(9)?,
                        schema_id: row.try_get(10)?,
                        storage_root: row.try_get(11)?,
                        storage_location: row.try_get(12)?,
                    })
                },
            mapper: |it| ListSchemas::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListSchemasParams<T1, T2>,
        ListSchemasQuery<'c, 'a, 's, C, ListSchemas, 3>,
        C,
    > for ListSchemasStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListSchemasParams<T1, T2>,
    ) -> ListSchemasQuery<'c, 'a, 's, C, ListSchemas, 3> {
        self.bind(
            client,
            &params.catalog_name,
            &params.page_token,
            &params.limit_value,
        )
    }
}
pub struct GetSchemaStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_schema() -> GetSchemaStmt {
    GetSchemaStmt(
        "SELECT schemas.name, catalogs.name AS catalog_name, COALESCE(schemas.comment, '') AS comment, schemas.properties, CONCAT(catalogs.name, '.', schemas.name) AS full_name, COALESCE(schemas.owner, '') AS owner, schemas.created_at, COALESCE(schemas.created_by, '') AS created_by, schemas.updated_at, COALESCE(schemas.updated_by, '') AS updated_by, schemas.id AS schema_id, COALESCE(schemas.storage_root, '') AS storage_root, COALESCE(schemas.storage_location, '') AS storage_location FROM uc_schemas schemas JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1 AND schemas.name = $2",
        None,
    )
}
impl GetSchemaStmt {
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
        catalog_name: &'a T1,
        name: &'a T2,
    ) -> GetSchemaQuery<'c, 'a, 's, C, GetSchema, 2> {
        GetSchemaQuery {
            client,
            params: [catalog_name, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<GetSchemaBorrowed, tokio_postgres::Error> {
                    Ok(GetSchemaBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        comment: row.try_get(2)?,
                        properties: row.try_get(3)?,
                        full_name: row.try_get(4)?,
                        owner: row.try_get(5)?,
                        created_at: row.try_get(6)?,
                        created_by: row.try_get(7)?,
                        updated_at: row.try_get(8)?,
                        updated_by: row.try_get(9)?,
                        schema_id: row.try_get(10)?,
                        storage_root: row.try_get(11)?,
                        storage_location: row.try_get(12)?,
                    })
                },
            mapper: |it| GetSchema::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetSchemaParams<T1, T2>,
        GetSchemaQuery<'c, 'a, 's, C, GetSchema, 2>,
        C,
    > for GetSchemaStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetSchemaParams<T1, T2>,
    ) -> GetSchemaQuery<'c, 'a, 's, C, GetSchema, 2> {
        self.bind(client, &params.catalog_name, &params.name)
    }
}
pub struct UpdateSchemaStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_schema() -> UpdateSchemaStmt {
    UpdateSchemaStmt(
        "UPDATE uc_schemas schemas SET name = COALESCE($1, schemas.name), comment = COALESCE($2, schemas.comment), properties = COALESCE($3, schemas.properties), updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint, updated_by = 'system' FROM uc_catalogs catalogs WHERE catalogs.id = schemas.catalog_id AND catalogs.name = $4 AND schemas.name = $5 RETURNING schemas.name, catalogs.name AS catalog_name, COALESCE(schemas.comment, '') AS comment, schemas.properties, CONCAT(catalogs.name, '.', schemas.name) AS full_name, COALESCE(schemas.owner, '') AS owner, schemas.created_at, COALESCE(schemas.created_by, '') AS created_by, schemas.updated_at, COALESCE(schemas.updated_by, '') AS updated_by, schemas.id AS schema_id, COALESCE(schemas.storage_root, '') AS storage_root, COALESCE(schemas.storage_location, '') AS storage_location",
        None,
    )
}
impl UpdateSchemaStmt {
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
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        new_name: &'a T1,
        comment: &'a T2,
        properties: &'a T3,
        catalog_name: &'a T4,
        name: &'a T5,
    ) -> UpdateSchemaQuery<'c, 'a, 's, C, UpdateSchema, 5> {
        UpdateSchemaQuery {
            client,
            params: [new_name, comment, properties, catalog_name, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<UpdateSchemaBorrowed, tokio_postgres::Error> {
                    Ok(UpdateSchemaBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        comment: row.try_get(2)?,
                        properties: row.try_get(3)?,
                        full_name: row.try_get(4)?,
                        owner: row.try_get(5)?,
                        created_at: row.try_get(6)?,
                        created_by: row.try_get(7)?,
                        updated_at: row.try_get(8)?,
                        updated_by: row.try_get(9)?,
                        schema_id: row.try_get(10)?,
                        storage_root: row.try_get(11)?,
                        storage_location: row.try_get(12)?,
                    })
                },
            mapper: |it| UpdateSchema::from(it),
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
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        UpdateSchemaParams<T1, T2, T3, T4, T5>,
        UpdateSchemaQuery<'c, 'a, 's, C, UpdateSchema, 5>,
        C,
    > for UpdateSchemaStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a UpdateSchemaParams<T1, T2, T3, T4, T5>,
    ) -> UpdateSchemaQuery<'c, 'a, 's, C, UpdateSchema, 5> {
        self.bind(
            client,
            &params.new_name,
            &params.comment,
            &params.properties,
            &params.catalog_name,
            &params.name,
        )
    }
}
pub struct DeleteSchemaStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_schema() -> DeleteSchemaStmt {
    DeleteSchemaStmt(
        "DELETE FROM uc_schemas schemas USING uc_catalogs catalogs WHERE catalogs.id = schemas.catalog_id AND catalogs.name = $1 AND schemas.name = $2 RETURNING schemas.name",
        None,
    )
}
impl DeleteSchemaStmt {
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
        catalog_name: &'a T1,
        name: &'a T2,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [catalog_name, name],
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
        DeleteSchemaParams<T1, T2>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for DeleteSchemaStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DeleteSchemaParams<T1, T2>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.catalog_name, &params.name)
    }
}
