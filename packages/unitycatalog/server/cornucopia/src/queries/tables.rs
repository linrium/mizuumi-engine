// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct CreateTableParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::JsonSql,
    T8: crate::StringSql,
    T9: crate::StringSql,
    T10: crate::JsonSql,
    T11: crate::StringSql,
    T12: crate::JsonSql,
> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub table_id: T3,
    pub name: T4,
    pub table_type: T5,
    pub data_source_format: T6,
    pub columns: T7,
    pub storage_location: T8,
    pub comment: T9,
    pub properties: T10,
    pub view_definition: T11,
    pub view_dependencies: T12,
}
#[derive(Debug)]
pub struct ListTablesParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub page_token: T3,
    pub limit_value: i64,
}
#[derive(Debug)]
pub struct GetTableParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub name: T3,
}
#[derive(Debug)]
pub struct DeleteTableParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub name: T3,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateTable {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_type: String,
    pub data_source_format: String,
    pub columns: serde_json::Value,
    pub storage_location: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub table_id: String,
    pub view_definition: String,
    pub view_dependencies: serde_json::Value,
}
pub struct CreateTableBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub schema_name: &'a str,
    pub table_type: &'a str,
    pub data_source_format: &'a str,
    pub columns: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub storage_location: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub table_id: &'a str,
    pub view_definition: &'a str,
    pub view_dependencies: postgres_types::Json<&'a serde_json::value::RawValue>,
}
impl<'a> From<CreateTableBorrowed<'a>> for CreateTable {
    fn from(
        CreateTableBorrowed {
            name,
            catalog_name,
            schema_name,
            table_type,
            data_source_format,
            columns,
            storage_location,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            table_id,
            view_definition,
            view_dependencies,
        }: CreateTableBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            table_type: table_type.into(),
            data_source_format: data_source_format.into(),
            columns: serde_json::from_str(columns.0.get()).unwrap(),
            storage_location: storage_location.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            table_id: table_id.into(),
            view_definition: view_definition.into(),
            view_dependencies: serde_json::from_str(view_dependencies.0.get()).unwrap(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListTables {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_type: String,
    pub data_source_format: String,
    pub columns: serde_json::Value,
    pub storage_location: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub table_id: String,
    pub view_definition: String,
    pub view_dependencies: serde_json::Value,
}
pub struct ListTablesBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub schema_name: &'a str,
    pub table_type: &'a str,
    pub data_source_format: &'a str,
    pub columns: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub storage_location: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub table_id: &'a str,
    pub view_definition: &'a str,
    pub view_dependencies: postgres_types::Json<&'a serde_json::value::RawValue>,
}
impl<'a> From<ListTablesBorrowed<'a>> for ListTables {
    fn from(
        ListTablesBorrowed {
            name,
            catalog_name,
            schema_name,
            table_type,
            data_source_format,
            columns,
            storage_location,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            table_id,
            view_definition,
            view_dependencies,
        }: ListTablesBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            table_type: table_type.into(),
            data_source_format: data_source_format.into(),
            columns: serde_json::from_str(columns.0.get()).unwrap(),
            storage_location: storage_location.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            table_id: table_id.into(),
            view_definition: view_definition.into(),
            view_dependencies: serde_json::from_str(view_dependencies.0.get()).unwrap(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetTable {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_type: String,
    pub data_source_format: String,
    pub columns: serde_json::Value,
    pub storage_location: String,
    pub comment: String,
    pub properties: serde_json::Value,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: i64,
    pub updated_by: String,
    pub table_id: String,
    pub view_definition: String,
    pub view_dependencies: serde_json::Value,
}
pub struct GetTableBorrowed<'a> {
    pub name: &'a str,
    pub catalog_name: &'a str,
    pub schema_name: &'a str,
    pub table_type: &'a str,
    pub data_source_format: &'a str,
    pub columns: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub storage_location: &'a str,
    pub comment: &'a str,
    pub properties: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub owner: &'a str,
    pub created_at: i64,
    pub created_by: &'a str,
    pub updated_at: i64,
    pub updated_by: &'a str,
    pub table_id: &'a str,
    pub view_definition: &'a str,
    pub view_dependencies: postgres_types::Json<&'a serde_json::value::RawValue>,
}
impl<'a> From<GetTableBorrowed<'a>> for GetTable {
    fn from(
        GetTableBorrowed {
            name,
            catalog_name,
            schema_name,
            table_type,
            data_source_format,
            columns,
            storage_location,
            comment,
            properties,
            owner,
            created_at,
            created_by,
            updated_at,
            updated_by,
            table_id,
            view_definition,
            view_dependencies,
        }: GetTableBorrowed<'a>,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            table_type: table_type.into(),
            data_source_format: data_source_format.into(),
            columns: serde_json::from_str(columns.0.get()).unwrap(),
            storage_location: storage_location.into(),
            comment: comment.into(),
            properties: serde_json::from_str(properties.0.get()).unwrap(),
            owner: owner.into(),
            created_at,
            created_by: created_by.into(),
            updated_at,
            updated_by: updated_by.into(),
            table_id: table_id.into(),
            view_definition: view_definition.into(),
            view_dependencies: serde_json::from_str(view_dependencies.0.get()).unwrap(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CreateTableQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CreateTableBorrowed, tokio_postgres::Error>,
    mapper: fn(CreateTableBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CreateTableQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CreateTableBorrowed) -> R,
    ) -> CreateTableQuery<'c, 'a, 's, C, R, N> {
        CreateTableQuery {
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
pub struct ListTablesQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListTablesBorrowed, tokio_postgres::Error>,
    mapper: fn(ListTablesBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListTablesQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListTablesBorrowed) -> R,
    ) -> ListTablesQuery<'c, 'a, 's, C, R, N> {
        ListTablesQuery {
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
pub struct GetTableQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GetTableBorrowed, tokio_postgres::Error>,
    mapper: fn(GetTableBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetTableQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(GetTableBorrowed) -> R) -> GetTableQuery<'c, 'a, 's, C, R, N> {
        GetTableQuery {
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
pub struct CreateTableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_table() -> CreateTableStmt {
    CreateTableStmt(
        "WITH parent_schema AS ( SELECT schemas.id, schemas.name AS schema_name, catalogs.name AS catalog_name FROM uc_schemas schemas JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1 AND schemas.name = $2 ), next_table AS ( SELECT COALESCE(NULLIF($3, ''), gen_random_uuid()::text) AS id, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint AS now_ms ), inserted AS ( INSERT INTO uc_tables ( id, schema_id, name, table_type, data_source_format, columns, storage_location, comment, properties, owner, created_at, created_by, updated_at, updated_by, view_definition, view_dependencies ) SELECT next_table.id, parent_schema.id, $4, $5, NULLIF($6, ''), COALESCE($7, '[]'::jsonb), NULLIF($8, ''), $9, COALESCE($10, '{}'::jsonb), 'system', next_table.now_ms, 'system', next_table.now_ms, 'system', NULLIF($11, ''), $12 FROM parent_schema, next_table RETURNING * ) SELECT inserted.name, parent_schema.catalog_name, parent_schema.schema_name, inserted.table_type, COALESCE(inserted.data_source_format, '') AS data_source_format, inserted.columns, COALESCE(inserted.storage_location, '') AS storage_location, COALESCE(inserted.comment, '') AS comment, inserted.properties, COALESCE(inserted.owner, '') AS owner, inserted.created_at, COALESCE(inserted.created_by, '') AS created_by, inserted.updated_at, COALESCE(inserted.updated_by, '') AS updated_by, inserted.id AS table_id, COALESCE(inserted.view_definition, '') AS view_definition, COALESCE(inserted.view_dependencies, 'null'::jsonb) AS view_dependencies FROM inserted JOIN parent_schema ON parent_schema.id = inserted.schema_id",
        None,
    )
}
impl CreateTableStmt {
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
        T7: crate::JsonSql,
        T8: crate::StringSql,
        T9: crate::StringSql,
        T10: crate::JsonSql,
        T11: crate::StringSql,
        T12: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        catalog_name: &'a T1,
        schema_name: &'a T2,
        table_id: &'a T3,
        name: &'a T4,
        table_type: &'a T5,
        data_source_format: &'a T6,
        columns: &'a T7,
        storage_location: &'a T8,
        comment: &'a T9,
        properties: &'a T10,
        view_definition: &'a T11,
        view_dependencies: &'a T12,
    ) -> CreateTableQuery<'c, 'a, 's, C, CreateTable, 12> {
        CreateTableQuery {
            client,
            params: [
                catalog_name,
                schema_name,
                table_id,
                name,
                table_type,
                data_source_format,
                columns,
                storage_location,
                comment,
                properties,
                view_definition,
                view_dependencies,
            ],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CreateTableBorrowed, tokio_postgres::Error> {
                    Ok(CreateTableBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        schema_name: row.try_get(2)?,
                        table_type: row.try_get(3)?,
                        data_source_format: row.try_get(4)?,
                        columns: row.try_get(5)?,
                        storage_location: row.try_get(6)?,
                        comment: row.try_get(7)?,
                        properties: row.try_get(8)?,
                        owner: row.try_get(9)?,
                        created_at: row.try_get(10)?,
                        created_by: row.try_get(11)?,
                        updated_at: row.try_get(12)?,
                        updated_by: row.try_get(13)?,
                        table_id: row.try_get(14)?,
                        view_definition: row.try_get(15)?,
                        view_dependencies: row.try_get(16)?,
                    })
                },
            mapper: |it| CreateTable::from(it),
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
    T7: crate::JsonSql,
    T8: crate::StringSql,
    T9: crate::StringSql,
    T10: crate::JsonSql,
    T11: crate::StringSql,
    T12: crate::JsonSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CreateTableParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>,
        CreateTableQuery<'c, 'a, 's, C, CreateTable, 12>,
        C,
    > for CreateTableStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CreateTableParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>,
    ) -> CreateTableQuery<'c, 'a, 's, C, CreateTable, 12> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.table_id,
            &params.name,
            &params.table_type,
            &params.data_source_format,
            &params.columns,
            &params.storage_location,
            &params.comment,
            &params.properties,
            &params.view_definition,
            &params.view_dependencies,
        )
    }
}
pub struct ListTablesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_tables() -> ListTablesStmt {
    ListTablesStmt(
        "SELECT tables.name, catalogs.name AS catalog_name, schemas.name AS schema_name, tables.table_type, COALESCE(tables.data_source_format, '') AS data_source_format, tables.columns, COALESCE(tables.storage_location, '') AS storage_location, COALESCE(tables.comment, '') AS comment, tables.properties, COALESCE(tables.owner, '') AS owner, tables.created_at, COALESCE(tables.created_by, '') AS created_by, tables.updated_at, COALESCE(tables.updated_by, '') AS updated_by, tables.id AS table_id, COALESCE(tables.view_definition, '') AS view_definition, COALESCE(tables.view_dependencies, 'null'::jsonb) AS view_dependencies FROM uc_tables tables JOIN uc_schemas schemas ON schemas.id = tables.schema_id JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1 AND schemas.name = $2 AND ($3 = '' OR tables.name > $3) ORDER BY tables.name LIMIT $4",
        None,
    )
}
impl ListTablesStmt {
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
        page_token: &'a T3,
        limit_value: &'a i64,
    ) -> ListTablesQuery<'c, 'a, 's, C, ListTables, 4> {
        ListTablesQuery {
            client,
            params: [catalog_name, schema_name, page_token, limit_value],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<ListTablesBorrowed, tokio_postgres::Error> {
                    Ok(ListTablesBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        schema_name: row.try_get(2)?,
                        table_type: row.try_get(3)?,
                        data_source_format: row.try_get(4)?,
                        columns: row.try_get(5)?,
                        storage_location: row.try_get(6)?,
                        comment: row.try_get(7)?,
                        properties: row.try_get(8)?,
                        owner: row.try_get(9)?,
                        created_at: row.try_get(10)?,
                        created_by: row.try_get(11)?,
                        updated_at: row.try_get(12)?,
                        updated_by: row.try_get(13)?,
                        table_id: row.try_get(14)?,
                        view_definition: row.try_get(15)?,
                        view_dependencies: row.try_get(16)?,
                    })
                },
            mapper: |it| ListTables::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListTablesParams<T1, T2, T3>,
        ListTablesQuery<'c, 'a, 's, C, ListTables, 4>,
        C,
    > for ListTablesStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListTablesParams<T1, T2, T3>,
    ) -> ListTablesQuery<'c, 'a, 's, C, ListTables, 4> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.page_token,
            &params.limit_value,
        )
    }
}
pub struct GetTableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_table() -> GetTableStmt {
    GetTableStmt(
        "SELECT tables.name, catalogs.name AS catalog_name, schemas.name AS schema_name, tables.table_type, COALESCE(tables.data_source_format, '') AS data_source_format, tables.columns, COALESCE(tables.storage_location, '') AS storage_location, COALESCE(tables.comment, '') AS comment, tables.properties, COALESCE(tables.owner, '') AS owner, tables.created_at, COALESCE(tables.created_by, '') AS created_by, tables.updated_at, COALESCE(tables.updated_by, '') AS updated_by, tables.id AS table_id, COALESCE(tables.view_definition, '') AS view_definition, COALESCE(tables.view_dependencies, 'null'::jsonb) AS view_dependencies FROM uc_tables tables JOIN uc_schemas schemas ON schemas.id = tables.schema_id JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1 AND schemas.name = $2 AND tables.name = $3",
        None,
    )
}
impl GetTableStmt {
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
        name: &'a T3,
    ) -> GetTableQuery<'c, 'a, 's, C, GetTable, 3> {
        GetTableQuery {
            client,
            params: [catalog_name, schema_name, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<GetTableBorrowed, tokio_postgres::Error> {
                    Ok(GetTableBorrowed {
                        name: row.try_get(0)?,
                        catalog_name: row.try_get(1)?,
                        schema_name: row.try_get(2)?,
                        table_type: row.try_get(3)?,
                        data_source_format: row.try_get(4)?,
                        columns: row.try_get(5)?,
                        storage_location: row.try_get(6)?,
                        comment: row.try_get(7)?,
                        properties: row.try_get(8)?,
                        owner: row.try_get(9)?,
                        created_at: row.try_get(10)?,
                        created_by: row.try_get(11)?,
                        updated_at: row.try_get(12)?,
                        updated_by: row.try_get(13)?,
                        table_id: row.try_get(14)?,
                        view_definition: row.try_get(15)?,
                        view_dependencies: row.try_get(16)?,
                    })
                },
            mapper: |it| GetTable::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetTableParams<T1, T2, T3>,
        GetTableQuery<'c, 'a, 's, C, GetTable, 3>,
        C,
    > for GetTableStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetTableParams<T1, T2, T3>,
    ) -> GetTableQuery<'c, 'a, 's, C, GetTable, 3> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.name,
        )
    }
}
pub struct DeleteTableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_table() -> DeleteTableStmt {
    DeleteTableStmt(
        "DELETE FROM uc_tables tables USING uc_schemas schemas, uc_catalogs catalogs WHERE schemas.id = tables.schema_id AND catalogs.id = schemas.catalog_id AND catalogs.name = $1 AND schemas.name = $2 AND tables.name = $3 RETURNING tables.name",
        None,
    )
}
impl DeleteTableStmt {
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
        name: &'a T3,
    ) -> StringQuery<'c, 'a, 's, C, String, 3> {
        StringQuery {
            client,
            params: [catalog_name, schema_name, name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        DeleteTableParams<T1, T2, T3>,
        StringQuery<'c, 'a, 's, C, String, 3>,
        C,
    > for DeleteTableStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a DeleteTableParams<T1, T2, T3>,
    ) -> StringQuery<'c, 'a, 's, C, String, 3> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.name,
        )
    }
}
