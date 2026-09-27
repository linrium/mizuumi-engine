use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{
    app_state::AppState,
    error::AppError,
    features::auth::{AuthenticatedPrincipal, resource_authorization as authorization},
};

use super::dtos::{
    CreateStagingTableRequest, CreateTableRequest, GetTableRequest, ListTablesRequest,
    ListTablesResponse, StagingTableInfo, TableInfo,
};

const TABLES_PATH: &str = "/api/2.1/unity-catalog/tables";

pub fn table_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/2.1/unity-catalog/staging-tables",
            post(create_staging_table),
        )
        .route(TABLES_PATH, post(create_table).get(list_tables))
        .route(
            &format!("{TABLES_PATH}/{{full_name}}"),
            get(get_table).delete(delete_table),
        )
}

async fn create_table(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<CreateTableRequest>>,
) -> Result<Json<TableInfo>, AppError> {
    Ok(Json(
        state.tables.create_table(request, principal.id).await?,
    ))
}

async fn create_staging_table(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<CreateStagingTableRequest>>,
) -> Result<Json<StagingTableInfo>, AppError> {
    Ok(Json(
        state
            .tables
            .create_staging_table(request, principal.id)
            .await?,
    ))
}

async fn list_tables(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Query(request)): Valid<Query<ListTablesRequest>>,
) -> Result<Json<ListTablesResponse>, AppError> {
    let catalog = request.catalog_name.clone();
    let schema = request.schema_name.clone();
    let mut response = state.tables.list_tables(request).await?;
    let mut visible = Vec::new();
    for table in response.tables {
        if authorization::can_read_table(&state, &principal, &catalog, &schema, &table.name).await?
        {
            visible.push(table);
        }
    }
    response.tables = visible;
    Ok(Json(response))
}

async fn get_table(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(full_name): Path<String>,
    Valid(Query(request)): Valid<Query<GetTableRequest>>,
) -> Result<Json<TableInfo>, AppError> {
    let (catalog, schema, table) = split_table_name(&full_name)?;
    authorization::require_allowed(
        authorization::can_read_table(&state, &principal, catalog, schema, table).await?,
        "read table",
    )
    .await?;
    Ok(Json(state.tables.get_table(full_name, request).await?))
}

async fn delete_table(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(full_name): Path<String>,
) -> Result<StatusCode, AppError> {
    let (catalog, schema, table) = split_table_name(&full_name)?;
    authorization::require_allowed(
        authorization::can_delete_table(&state, &principal, catalog, schema, table).await?,
        "delete table",
    )
    .await?;
    state.tables.delete_table(full_name).await?;
    Ok(StatusCode::OK)
}

fn split_table_name(full_name: &str) -> Result<(&str, &str, &str), AppError> {
    let mut parts = full_name.split('.');
    let catalog = parts.next().unwrap_or_default();
    let schema = parts.next().unwrap_or_default();
    let table = parts.next().unwrap_or_default();
    if catalog.is_empty() || schema.is_empty() || table.is_empty() || parts.next().is_some() {
        return Err(AppError::InvalidParameter(format!(
            "invalid table full name: {full_name}"
        )));
    }
    Ok((catalog, schema, table))
}
