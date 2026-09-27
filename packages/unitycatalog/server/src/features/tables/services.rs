use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::{
    client::GenericClient,
    queries::{schemas as schema_queries, tables as table_queries},
};

use crate::error::AppError;

use super::{
    dtos::{
        CreateStagingTableRequest, CreateTableRequest, GetTableRequest, ListTablesRequest,
        ListTablesResponse, StagingTableInfo, TableInfo,
    },
    models::Table,
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";
const TABLE_TYPE_EXTERNAL: &str = "EXTERNAL";
const TABLE_TYPE_MANAGED: &str = "MANAGED";

#[async_trait]
pub trait TableService: Send + Sync {
    async fn create_table(
        &self,
        request: CreateTableRequest,
        principal: String,
    ) -> Result<TableInfo, AppError>;
    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        principal: String,
    ) -> Result<StagingTableInfo, AppError>;
    async fn is_staging_table_owner(
        &self,
        staging_id: String,
        principal: String,
    ) -> Result<bool, AppError>;
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
    async fn create_table(
        &self,
        mut request: CreateTableRequest,
        principal: String,
    ) -> Result<TableInfo, AppError> {
        validate_create_table(&request)?;
        normalize_columns(&mut request);

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        ensure_schema_exists(&transaction, &request.catalog_name, &request.schema_name).await?;

        let columns = serde_json::to_value(request.columns)?;
        let properties = serde_json::to_value(request.properties.unwrap_or_default())?;
        let storage_location = request.storage_location.unwrap_or_default();
        let data_source_format = request.data_source_format.unwrap_or_else(|| {
            if request.table_type == TABLE_TYPE_MANAGED {
                "DELTA".to_string()
            } else {
                String::new()
            }
        });
        let comment = request.comment.unwrap_or_default();
        let view_definition = request.view_definition.unwrap_or_default();
        let view_dependencies = request.view_dependencies.unwrap_or(Value::Null);

        let staging_id = if request.table_type == TABLE_TYPE_MANAGED {
            let staging = transaction
                .query_opt(
                    "SELECT staging.id, staging.name, staging.created_by, staging.finalized_at \
                     FROM uc_staging_tables staging \
                     JOIN uc_schemas schemas ON schemas.id = staging.schema_id \
                     JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id \
                     WHERE staging.staging_location = $1 \
                       AND catalogs.name = $2 AND schemas.name = $3 \
                     FOR UPDATE OF staging",
                    &[
                        &storage_location,
                        &request.catalog_name,
                        &request.schema_name,
                    ],
                )
                .await?
                .ok_or_else(|| {
                    AppError::FailedPrecondition(
                        "managed table requires a matching staging table".to_string(),
                    )
                })?;
            if staging.get::<_, String>("name") != request.name {
                return Err(AppError::InvalidParameter(
                    "managed table name must match its staging table".to_string(),
                ));
            }
            if staging.get::<_, String>("created_by") != principal {
                return Err(AppError::Forbidden(
                    "staging table belongs to another principal".to_string(),
                ));
            }
            if staging.get::<_, Option<i64>>("finalized_at").is_some() {
                return Err(AppError::FailedPrecondition(
                    "staging table has already been finalized".to_string(),
                ));
            }
            Some(staging.get::<_, String>("id"))
        } else {
            None
        };
        let table_id = staging_id.clone().unwrap_or_default();

        let row = table_queries::create_table()
            .bind(
                &transaction,
                &request.catalog_name,
                &request.schema_name,
                &table_id,
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

        // The creator must be able to load and write the table after creation.
        let mut table = row_to_table(row)?;
        transaction
            .execute(
                "UPDATE uc_tables SET owner = $2, created_by = $2, updated_by = $2 WHERE id = $1",
                &[&table.table_id, &principal],
            )
            .await?;
        transaction
            .execute(
                "INSERT INTO uc_permissions (principal, resource_id, securable_type, privilege) \
                 VALUES ($1, $2, 'table', 'OWNER') ON CONFLICT DO NOTHING",
                &[&principal, &table.table_id],
            )
            .await?;
        table.owner = principal.clone();
        table.created_by = principal.clone();
        table.updated_by = principal;

        if let Some(staging_id) = staging_id {
            transaction
                .execute(
                    "UPDATE uc_staging_tables \
                     SET finalized_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint \
                     WHERE id = $1",
                    &[&staging_id],
                )
                .await?;
        }

        transaction.commit().await?;

        Ok(table.into_info(false, false))
    }

    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        principal: String,
    ) -> Result<StagingTableInfo, AppError> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let parent = transaction
            .query_opt(
                "SELECT schemas.id AS schema_id, \
                        COALESCE(schemas.storage_location, catalogs.storage_location) AS storage_location \
                 FROM uc_schemas schemas \
                 JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id \
                 WHERE catalogs.name = $1 AND schemas.name = $2",
                &[&request.catalog_name, &request.schema_name],
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "schema {}.{}",
                    request.catalog_name, request.schema_name
                ))
            })?;
        let root: Option<String> = parent.get("storage_location");
        let root = root.filter(|value| !value.is_empty()).ok_or_else(|| {
            AppError::FailedPrecondition(
                "schema or catalog must have a managed storage location".to_string(),
            )
        })?;
        let schema_id: String = parent.get("schema_id");
        let already_exists: bool = transaction
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM uc_tables WHERE schema_id = $1 AND name = $2)",
                &[&schema_id, &request.name],
            )
            .await?
            .get(0);
        if already_exists {
            return Err(AppError::Conflict("table name".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let staging_location = format!("{}/tables/{id}", root.trim_end_matches('/'));
        transaction
            .execute(
                "INSERT INTO uc_staging_tables \
                 (id, schema_id, name, staging_location, created_by, created_at) \
                 VALUES ($1, $2, $3, $4, $5, (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint)",
                &[&id, &schema_id, &request.name, &staging_location, &principal],
            )
            .await?;
        transaction.commit().await?;

        Ok(StagingTableInfo {
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            id,
            staging_location,
        })
    }

    async fn is_staging_table_owner(
        &self,
        staging_id: String,
        principal: String,
    ) -> Result<bool, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM uc_staging_tables \
                 WHERE id = $1 AND created_by = $2 AND finalized_at IS NULL)",
                &[&staging_id, &principal],
            )
            .await?;
        Ok(row.get(0))
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

async fn ensure_schema_exists<C: GenericClient>(
    client: &C,
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
    let storage_location = request.storage_location.as_deref().unwrap_or_default();
    match request.table_type.as_str() {
        TABLE_TYPE_EXTERNAL => {
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
        }
        TABLE_TYPE_MANAGED => {
            if storage_location.is_empty() {
                return Err(AppError::InvalidParameter(
                    "storage_location from a staging table is required for managed table"
                        .to_string(),
                ));
            }
        }
        "STREAMING_TABLE" | "MATERIALIZED_VIEW" | "METRIC_VIEW" | "VIEW" => {
            if request.view_definition.as_deref().is_none_or(str::is_empty) {
                return Err(AppError::InvalidParameter(format!(
                    "view_definition is required for {}",
                    request.table_type
                )));
            }
            if !storage_location.is_empty() {
                return Err(AppError::InvalidParameter(format!(
                    "storage_location is not supported for {}",
                    request.table_type
                )));
            }
        }
        _ => {
            return Err(AppError::InvalidParameter(format!(
                "unsupported table_type: {}",
                request.table_type
            )));
        }
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
