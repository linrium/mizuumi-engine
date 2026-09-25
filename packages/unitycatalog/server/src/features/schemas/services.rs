use std::collections::BTreeMap;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::{catalogs as catalog_queries, schemas as schema_queries};

use crate::error::AppError;

use super::{
    dtos::{
        CreateSchemaRequest, DeleteSchemaRequest, ListSchemasRequest, ListSchemasResponse,
        SchemaInfo, UpdateSchemaRequest,
    },
    models::Schema,
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";

#[async_trait]
pub trait SchemaService: Send + Sync {
    async fn create_schema(&self, request: CreateSchemaRequest) -> Result<SchemaInfo, AppError>;
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
    ) -> Result<ListSchemasResponse, AppError>;
    async fn get_schema(&self, full_name: String) -> Result<SchemaInfo, AppError>;
    async fn update_schema(
        &self,
        full_name: String,
        request: UpdateSchemaRequest,
    ) -> Result<SchemaInfo, AppError>;
    async fn delete_schema(
        &self,
        full_name: String,
        request: DeleteSchemaRequest,
    ) -> Result<(), AppError>;
}

pub struct DefaultSchemaService {
    pool: Pool,
}

impl DefaultSchemaService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchemaService for DefaultSchemaService {
    async fn create_schema(&self, request: CreateSchemaRequest) -> Result<SchemaInfo, AppError> {
        let client = self.pool.get().await?;
        ensure_catalog_exists(&client, &request.catalog_name).await?;
        let properties = serde_json::to_value(request.properties.unwrap_or_default())?;
        let comment = request.comment.unwrap_or_default();
        let storage_root = request.storage_root.unwrap_or_default();
        validate_storage_root(&storage_root)?;

        let row = schema_queries::create_schema()
            .bind(
                &client,
                &request.catalog_name,
                &request.name,
                &comment,
                &properties,
                &storage_root,
            )
            .opt()
            .await
            .map_err(map_schema_write_error)?
            .ok_or_else(|| AppError::NotFound(format!("catalog {}", request.catalog_name)))?;

        Ok(row_to_schema(row)?.into())
    }

    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
    ) -> Result<ListSchemasResponse, AppError> {
        let client = self.pool.get().await?;
        ensure_catalog_exists(&client, &request.catalog_name).await?;
        let page_size = page_size(request.max_results)?;
        let limit = i64::from(page_size);
        let page_token = request.page_token.unwrap_or_default();

        let schemas = schema_queries::list_schemas()
            .bind(&client, &request.catalog_name, &page_token, &limit)
            .all()
            .await?
            .into_iter()
            .map(row_to_schema)
            .collect::<Result<Vec<_>, _>>()?;

        let next_page_token = if schemas.len() == page_size as usize {
            schemas.last().map(|schema| schema.name.clone())
        } else {
            None
        };

        Ok(ListSchemasResponse {
            schemas: schemas.into_iter().map(Into::into).collect(),
            next_page_token,
        })
    }

    async fn get_schema(&self, full_name: String) -> Result<SchemaInfo, AppError> {
        let names = split_schema_full_name(&full_name)?;
        let client = self.pool.get().await?;
        let row = schema_queries::get_schema()
            .bind(&client, &names.catalog_name, &names.schema_name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("schema {full_name}")))?;

        Ok(row_to_schema(row)?.into())
    }

    async fn update_schema(
        &self,
        full_name: String,
        request: UpdateSchemaRequest,
    ) -> Result<SchemaInfo, AppError> {
        let names = split_schema_full_name(&full_name)?;
        let client = self.pool.get().await?;
        let current = schema_queries::get_schema()
            .bind(&client, &names.catalog_name, &names.schema_name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("schema {full_name}")))?;
        let current = row_to_schema(current)?;

        if request.new_name.is_none()
            && request.comment.is_none()
            && request.properties.as_ref().is_none_or(BTreeMap::is_empty)
        {
            return Ok(current.into());
        }

        let new_name = request.new_name.unwrap_or_else(|| current.name.clone());
        let comment = request.comment.unwrap_or(current.comment);
        let properties = match request.properties {
            Some(properties) if !properties.is_empty() => serde_json::to_value(properties)?,
            None => serde_json::to_value(current.properties)?,
            Some(_) => serde_json::to_value(current.properties)?,
        };

        let row = schema_queries::update_schema()
            .bind(
                &client,
                &new_name,
                &comment,
                &properties,
                &names.catalog_name,
                &names.schema_name,
            )
            .one()
            .await
            .map_err(map_schema_write_error)?;

        Ok(row_to_schema(row)?.into())
    }

    async fn delete_schema(
        &self,
        full_name: String,
        request: DeleteSchemaRequest,
    ) -> Result<(), AppError> {
        let names = split_schema_full_name(&full_name)?;
        let _force = request.force.unwrap_or(false);
        let client = self.pool.get().await?;
        schema_queries::delete_schema()
            .bind(&client, &names.catalog_name, &names.schema_name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("schema {full_name}")))?;

        Ok(())
    }
}

async fn ensure_catalog_exists(
    client: &deadpool_postgres::Client,
    catalog_name: &str,
) -> Result<(), AppError> {
    catalog_queries::get_catalog()
        .bind(client, &catalog_name)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("catalog {catalog_name}")))?;

    Ok(())
}

fn page_size(max_results: Option<i32>) -> Result<i32, AppError> {
    match max_results {
        Some(value) if value < 0 => Err(AppError::InvalidParameter(
            "max_results must be greater than or equal to 0".to_string(),
        )),
        Some(0) | None => Ok(DEFAULT_PAGE_SIZE),
        Some(value) => Ok(value.min(DEFAULT_PAGE_SIZE)),
    }
}

fn split_schema_full_name(full_name: &str) -> Result<SchemaFullName, AppError> {
    let mut parts = full_name.split('.');
    let catalog_name = parts.next().unwrap_or_default();
    let schema_name = parts.next().unwrap_or_default();

    if catalog_name.is_empty() || schema_name.is_empty() || parts.next().is_some() {
        return Err(AppError::InvalidParameter(format!(
            "invalid schema full name: {full_name}"
        )));
    }

    Ok(SchemaFullName {
        catalog_name: catalog_name.to_string(),
        schema_name: schema_name.to_string(),
    })
}

fn validate_storage_root(storage_root: &str) -> Result<(), AppError> {
    if storage_root.contains(MANAGED_STORAGE_PREFIX) {
        return Err(AppError::InvalidParameter(format!(
            "input path '{storage_root}' contains managed storage prefix {MANAGED_STORAGE_PREFIX}"
        )));
    }

    Ok(())
}

fn map_schema_write_error(error: tokio_postgres::Error) -> AppError {
    match error.code() {
        Some(&SqlState::UNIQUE_VIOLATION) => AppError::Conflict("schema name".to_string()),
        Some(&SqlState::FOREIGN_KEY_VIOLATION) => AppError::NotFound("catalog".to_string()),
        _ => AppError::Postgres(error),
    }
}

fn row_to_schema(row: impl IntoSchemaParts) -> Result<Schema, AppError> {
    let row = row.into_schema_parts();
    let properties = serde_json::from_value(row.properties)?;

    Ok(Schema {
        name: row.name,
        catalog_name: row.catalog_name,
        comment: row.comment,
        properties,
        full_name: row.full_name,
        owner: row.owner,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: (row.updated_at > 0).then_some(row.updated_at),
        updated_by: row.updated_by,
        schema_id: row.schema_id,
        storage_root: row.storage_root,
        storage_location: row.storage_location,
    })
}

struct SchemaFullName {
    catalog_name: String,
    schema_name: String,
}

struct SchemaParts {
    name: String,
    catalog_name: String,
    comment: String,
    properties: Value,
    full_name: String,
    owner: String,
    created_at: i64,
    created_by: String,
    updated_at: i64,
    updated_by: String,
    schema_id: String,
    storage_root: String,
    storage_location: String,
}

trait IntoSchemaParts {
    fn into_schema_parts(self) -> SchemaParts;
}

macro_rules! impl_into_schema_parts {
    ($type:ty) => {
        impl IntoSchemaParts for $type {
            fn into_schema_parts(self) -> SchemaParts {
                SchemaParts {
                    name: self.name,
                    catalog_name: self.catalog_name,
                    comment: self.comment,
                    properties: self.properties,
                    full_name: self.full_name,
                    owner: self.owner,
                    created_at: self.created_at,
                    created_by: self.created_by,
                    updated_at: self.updated_at,
                    updated_by: self.updated_by,
                    schema_id: self.schema_id,
                    storage_root: self.storage_root,
                    storage_location: self.storage_location,
                }
            }
        }
    };
}

impl_into_schema_parts!(schema_queries::CreateSchema);
impl_into_schema_parts!(schema_queries::ListSchemas);
impl_into_schema_parts!(schema_queries::GetSchema);
impl_into_schema_parts!(schema_queries::UpdateSchema);
