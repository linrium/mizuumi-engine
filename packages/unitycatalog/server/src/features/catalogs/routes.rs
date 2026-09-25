use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CatalogInfo, CreateCatalogRequest, DeleteCatalogRequest, ListCatalogsRequest,
    ListCatalogsResponse, UpdateCatalogRequest,
};

const CATALOGS_PATH: &str = "/api/2.1/unity-catalog/catalogs";

pub fn catalog_router() -> Router<AppState> {
    Router::new()
        .route(CATALOGS_PATH, post(create_catalog).get(list_catalogs))
        .route(
            &format!("{CATALOGS_PATH}/{{name}}"),
            get(get_catalog)
                .patch(update_catalog)
                .delete(delete_catalog),
        )
}

async fn create_catalog(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<CreateCatalogRequest>>,
) -> Result<Json<CatalogInfo>, AppError> {
    Ok(Json(state.catalogs.create_catalog(request).await?))
}

async fn list_catalogs(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListCatalogsRequest>>,
) -> Result<Json<ListCatalogsResponse>, AppError> {
    Ok(Json(state.catalogs.list_catalogs(request).await?))
}

async fn get_catalog(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CatalogInfo>, AppError> {
    Ok(Json(state.catalogs.get_catalog(name).await?))
}

async fn update_catalog(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Json(request)): Valid<Json<UpdateCatalogRequest>>,
) -> Result<Json<CatalogInfo>, AppError> {
    Ok(Json(state.catalogs.update_catalog(name, request).await?))
}

async fn delete_catalog(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Query(request)): Valid<Query<DeleteCatalogRequest>>,
) -> Result<StatusCode, AppError> {
    state.catalogs.delete_catalog(name, request).await?;
    Ok(StatusCode::OK)
}
