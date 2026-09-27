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
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Query(request)): Valid<Query<ListSchemasRequest>>,
) -> Result<Json<ListSchemasResponse>, AppError> {
    let catalog_name = request.catalog_name.clone();
    let mut response = state.schemas.list_schemas(request).await?;
    let mut visible = Vec::new();
    for schema in response.schemas {
        if authorization::can_read_schema(&state, &principal, &catalog_name, &schema.name).await? {
            visible.push(schema);
        }
    }
    response.schemas = visible;
    Ok(Json(response))
}

async fn get_schema(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(full_name): Path<String>,
) -> Result<Json<SchemaInfo>, AppError> {
    let (catalog, schema) = split_schema_name(&full_name)?;
    authorization::require_allowed(
        authorization::can_read_schema(&state, &principal, catalog, schema).await?,
        "read schema",
    )
    .await?;
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
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(full_name): Path<String>,
    Valid(Query(request)): Valid<Query<DeleteSchemaRequest>>,
) -> Result<StatusCode, AppError> {
    let (catalog, schema) = split_schema_name(&full_name)?;
    authorization::require_allowed(
        authorization::can_delete_schema(&state, &principal, catalog, schema).await?,
        "delete schema",
    )
    .await?;
    state.schemas.delete_schema(full_name, request).await?;
    Ok(StatusCode::OK)
}

fn split_schema_name(full_name: &str) -> Result<(&str, &str), AppError> {
    let (catalog, schema) = full_name.split_once('.').ok_or_else(|| {
        AppError::InvalidParameter(format!("invalid schema full name: {full_name}"))
    })?;
    if catalog.is_empty() || schema.is_empty() || schema.contains('.') {
        return Err(AppError::InvalidParameter(format!(
            "invalid schema full name: {full_name}"
        )));
    }
    Ok((catalog, schema))
}
