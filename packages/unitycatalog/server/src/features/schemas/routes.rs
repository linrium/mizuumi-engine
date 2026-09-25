use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CreateSchemaRequest, DeleteSchemaRequest, ListSchemasRequest, ListSchemasResponse, SchemaInfo,
    UpdateSchemaRequest,
};

const SCHEMAS_PATH: &str = "/api/2.1/unity-catalog/schemas";

pub fn schema_router() -> Router<AppState> {
    Router::new()
        .route(SCHEMAS_PATH, post(create_schema).get(list_schemas))
        .route(
            &format!("{SCHEMAS_PATH}/{{full_name}}"),
            get(get_schema).patch(update_schema).delete(delete_schema),
        )
}

async fn create_schema(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<CreateSchemaRequest>>,
) -> Result<Json<SchemaInfo>, AppError> {
    Ok(Json(state.schemas.create_schema(request).await?))
}

async fn list_schemas(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListSchemasRequest>>,
) -> Result<Json<ListSchemasResponse>, AppError> {
    Ok(Json(state.schemas.list_schemas(request).await?))
}

async fn get_schema(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
) -> Result<Json<SchemaInfo>, AppError> {
    Ok(Json(state.schemas.get_schema(full_name).await?))
}

async fn update_schema(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
    Valid(Json(request)): Valid<Json<UpdateSchemaRequest>>,
) -> Result<Json<SchemaInfo>, AppError> {
    Ok(Json(state.schemas.update_schema(full_name, request).await?))
}

async fn delete_schema(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
    Valid(Query(request)): Valid<Query<DeleteSchemaRequest>>,
) -> Result<StatusCode, AppError> {
    state.schemas.delete_schema(full_name, request).await?;
    Ok(StatusCode::OK)
}
