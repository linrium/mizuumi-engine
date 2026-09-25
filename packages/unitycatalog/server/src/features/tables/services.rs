use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::{schemas as schema_queries, tables as table_queries};

use crate::error::AppError;

use super::{
    dtos::{CreateTableRequest, GetTableRequest, ListTablesRequest, ListTablesResponse, TableInfo},
    models::Table,
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";
const TABLE_TYPE_EXTERNAL: &str = "EXTERNAL";

#[async_trait]
pub trait TableService: Send + Sync {
    async fn create_table(&self, request: CreateTableRequest) -> Result<TableInfo, AppError>;
    async fn list_tables(&self, request: ListTablesRequest)
    -> Result<ListTablesResponse, AppError>;
    async fn get_table(
        &self,
        full_name: String,
        request: GetTableRequest,
    ) -> Result<TableInfo, AppError>;
    async fn delete_table(&self, full_name: String) -> Result<(), AppError>;
}

pub struct DefaultTableService {
    pool: Pool,
}

impl DefaultTableService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TableService for DefaultTableService {
    async fn create_table(&self, mut request: CreateTableRequest) -> Result<TableInfo, AppError> {
        validate_create_table(&request)?;
        normalize_columns(&mut request);

        let client = self.pool.get().await?;
        ensure_schema_exists(&client, &request.catalog_name, &request.schema_name).await?;

        let columns = serde_json::to_value(request.columns)?;
        let properties = serde_json::to_value(request.properties.unwrap_or_default())?;
        let storage_location = request.storage_location.unwrap_or_default();
        let data_source_format = request.data_source_format.unwrap_or_default();
        let comment = request.comment.unwrap_or_default();
        let view_definition = request.view_definition.unwrap_or_default();
        let view_dependencies = request.view_dependencies.unwrap_or(Value::Null);

        let row = table_queries::create_table()
            .bind(
                &client,
                &request.catalog_name,
                &request.schema_name,
                &request.name,
                &request.table_type,
                &data_source_format,
                &columns,
                &storage_location,
                &comment,
                &properties,
                &view_definition,
                &view_dependencies,
            )
            .opt()
            .await
            .map_err(map_table_write_error)?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "schema {}.{}",
                    request.catalog_name, request.schema_name
                ))
            })?;

        Ok(row_to_table(row)?.into_info(false, false))
    }

    async fn list_tables(
        &self,
        request: ListTablesRequest,
    ) -> Result<ListTablesResponse, AppError> {
        let client = self.pool.get().await?;
        ensure_schema_exists(&client, &request.catalog_name, &request.schema_name).await?;
        let page_size = page_size(request.max_results)?;
        let limit = i64::from(page_size);
        let page_token = request.page_token.unwrap_or_default();
        let omit_properties = request.omit_properties.unwrap_or(false);
        let omit_columns = request.omit_columns.unwrap_or(false);

        let tables = table_queries::list_tables()
            .bind(
                &client,
                &request.catalog_name,
                &request.schema_name,
                &page_token,
                &limit,
            )
            .all()
            .await?
            .into_iter()
            .map(row_to_table)
            .collect::<Result<Vec<_>, _>>()?;

        let next_page_token = if tables.len() == page_size as usize {
            tables.last().map(|table| table.name.clone())
        } else {
            None
        };

        Ok(ListTablesResponse {
            tables: tables
                .into_iter()
                .map(|table| table.into_info(omit_properties, omit_columns))
                .collect(),
            next_page_token,
        })
    }

    async fn get_table(
        &self,
        full_name: String,
        request: GetTableRequest,
    ) -> Result<TableInfo, AppError> {
        let _read_streaming_table_as_managed = request.read_streaming_table_as_managed;
        let _read_materialized_view_as_managed = request.read_materialized_view_as_managed;
        let names = split_table_full_name(&full_name)?;
        let client = self.pool.get().await?;
        let row = table_queries::get_table()
            .bind(
                &client,
                &names.catalog_name,
                &names.schema_name,
                &names.table_name,
            )
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("table {full_name}")))?;

        Ok(row_to_table(row)?.into_info(false, false))
    }

    async fn delete_table(&self, full_name: String) -> Result<(), AppError> {
        let names = split_table_full_name(&full_name)?;
        let client = self.pool.get().await?;
        table_queries::delete_table()
            .bind(
                &client,
                &names.catalog_name,
                &names.schema_name,
                &names.table_name,
            )
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("table {full_name}")))?;

        Ok(())
    }
}

async fn ensure_schema_exists(
    client: &deadpool_postgres::Client,
    catalog_name: &str,
    schema_name: &str,
) -> Result<(), AppError> {
    schema_queries::get_schema()
        .bind(client, &catalog_name, &schema_name)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("schema {catalog_name}.{schema_name}")))?;

    Ok(())
}

fn validate_create_table(request: &CreateTableRequest) -> Result<(), AppError> {
    if request.table_type != TABLE_TYPE_EXTERNAL {
        return Err(AppError::InvalidParameter(
            "only EXTERNAL table creation is supported".to_string(),
        ));
    }

    let storage_location = request.storage_location.as_deref().unwrap_or_default();
    if storage_location.is_empty() {
        return Err(AppError::InvalidParameter(
            "storage_location is required for external table".to_string(),
        ));
    }
    if storage_location.contains(MANAGED_STORAGE_PREFIX) {
        return Err(AppError::InvalidParameter(format!(
            "input path '{storage_location}' contains managed storage prefix {MANAGED_STORAGE_PREFIX}"
        )));
    }

    if let Some(format) = &request.data_source_format {
        validate_data_source_format(format)?;
    }

    Ok(())
}

fn validate_data_source_format(format: &str) -> Result<(), AppError> {
    const FORMATS: &[&str] = &[
        "DELTA", "ICEBERG", "CSV", "JSON", "AVRO", "PARQUET", "ORC", "TEXT",
    ];
    if !FORMATS.contains(&format) {
        return Err(AppError::InvalidParameter(format!(
            "unsupported data_source_format: {format}"
        )));
    }

    Ok(())
}

fn normalize_columns(request: &mut CreateTableRequest) {
    for column in &mut request.columns {
        if let Some(type_text) = &mut column.type_text {
            *type_text = type_text.to_lowercase();
        }
    }
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

fn split_table_full_name(full_name: &str) -> Result<TableFullName, AppError> {
    let mut parts = full_name.split('.');
    let catalog_name = parts.next().unwrap_or_default();
    let schema_name = parts.next().unwrap_or_default();
    let table_name = parts.next().unwrap_or_default();

    if catalog_name.is_empty()
        || schema_name.is_empty()
        || table_name.is_empty()
        || parts.next().is_some()
    {
        return Err(AppError::InvalidParameter(format!(
            "invalid table name: {full_name}"
        )));
    }

    Ok(TableFullName {
        catalog_name: catalog_name.to_string(),
        schema_name: schema_name.to_string(),
        table_name: table_name.to_string(),
    })
}

fn map_table_write_error(error: tokio_postgres::Error) -> AppError {
    match error.code() {
        Some(&SqlState::UNIQUE_VIOLATION) => AppError::Conflict("table name".to_string()),
        Some(&SqlState::FOREIGN_KEY_VIOLATION) => AppError::NotFound("schema".to_string()),
        _ => AppError::Postgres(error),
    }
}

fn row_to_table(row: impl IntoTableParts) -> Result<Table, AppError> {
    let row = row.into_table_parts();
    let columns = serde_json::from_value(row.columns)?;
    let properties = serde_json::from_value(row.properties)?;
    let view_dependencies = (!row.view_dependencies.is_null()).then_some(row.view_dependencies);

    Ok(Table {
        name: row.name,
        catalog_name: row.catalog_name,
        schema_name: row.schema_name,
        table_type: row.table_type,
        data_source_format: row.data_source_format,
        columns,
        storage_location: row.storage_location,
        comment: row.comment,
        properties,
        owner: row.owner,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: (row.updated_at > 0).then_some(row.updated_at),
        updated_by: row.updated_by,
        table_id: row.table_id,
        view_definition: row.view_definition,
        view_dependencies,
    })
}

struct TableFullName {
    catalog_name: String,
    schema_name: String,
    table_name: String,
}

struct TableParts {
    name: String,
    catalog_name: String,
    schema_name: String,
    table_type: String,
    data_source_format: String,
    columns: Value,
    storage_location: String,
    comment: String,
    properties: Value,
    owner: String,
    created_at: i64,
    created_by: String,
    updated_at: i64,
    updated_by: String,
    table_id: String,
    view_definition: String,
    view_dependencies: Value,
}

trait IntoTableParts {
    fn into_table_parts(self) -> TableParts;
}

macro_rules! impl_into_table_parts {
    ($type:ty) => {
        impl IntoTableParts for $type {
            fn into_table_parts(self) -> TableParts {
                TableParts {
                    name: self.name,
                    catalog_name: self.catalog_name,
                    schema_name: self.schema_name,
                    table_type: self.table_type,
                    data_source_format: self.data_source_format,
                    columns: self.columns,
                    storage_location: self.storage_location,
                    comment: self.comment,
                    properties: self.properties,
                    owner: self.owner,
                    created_at: self.created_at,
                    created_by: self.created_by,
                    updated_at: self.updated_at,
                    updated_by: self.updated_by,
                    table_id: self.table_id,
                    view_definition: self.view_definition,
                    view_dependencies: self.view_dependencies,
                }
            }
        }
    };
}

impl_into_table_parts!(table_queries::CreateTable);
impl_into_table_parts!(table_queries::ListTables);
impl_into_table_parts!(table_queries::GetTable);
