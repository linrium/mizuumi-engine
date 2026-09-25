use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CreateTableRequest, GetTableRequest, ListTablesRequest, ListTablesResponse, TableInfo,
};

const TABLES_PATH: &str = "/api/2.1/unity-catalog/tables";

pub fn table_router() -> Router<AppState> {
    Router::new()
        .route(TABLES_PATH, post(create_table).get(list_tables))
        .route(
            &format!("{TABLES_PATH}/{{full_name}}"),
            get(get_table).delete(delete_table),
        )
}

async fn create_table(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<CreateTableRequest>>,
) -> Result<Json<TableInfo>, AppError> {
    Ok(Json(state.tables.create_table(request).await?))
}

async fn list_tables(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListTablesRequest>>,
) -> Result<Json<ListTablesResponse>, AppError> {
    Ok(Json(state.tables.list_tables(request).await?))
}

async fn get_table(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
    Valid(Query(request)): Valid<Query<GetTableRequest>>,
) -> Result<Json<TableInfo>, AppError> {
    Ok(Json(state.tables.get_table(full_name, request).await?))
}

async fn delete_table(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.tables.delete_table(full_name).await?;
    Ok(StatusCode::OK)
}
