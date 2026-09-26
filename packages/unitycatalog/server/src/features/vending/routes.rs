use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    GenerateTemporaryPathCredentialRequest, ListBucketsRequest, ListBucketsResponse,
    TemporaryCredentialsResponse,
};

pub fn vending_router() -> Router<AppState> {
    Router::new()
        .route("/api/vending/buckets", get(list_buckets))
        .route(
            "/api/2.1/unity-catalog/temporary-path-credentials",
            post(generate_temporary_path_credentials),
        )
}

async fn generate_temporary_path_credentials(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryPathCredentialRequest>>,
) -> Result<Json<TemporaryCredentialsResponse>, AppError> {
    Ok(Json(
        state
            .vending
            .generate_temporary_path_credentials(request)
            .await?,
    ))
}

async fn list_buckets(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListBucketsRequest>>,
) -> Result<Json<ListBucketsResponse>, AppError> {
    Ok(Json(state.vending.list_buckets(request).await?))
}
