use axum::{
    Json, Router,
    extract::{Extension, State},
    routing::get,
};
use axum_valid::Valid;
use serde_json::{Value, json};

use crate::{
    app_state::AppState,
    error::AppError,
    features::auth::{AuthenticatedPrincipal, resource_authorization as authorization},
};

use super::dtos::{DeltaCommitRequest, DeltaGetCommitsRequest, DeltaGetCommitsResponse};

pub fn delta_commits_router() -> Router<AppState> {
    Router::new().route(
        "/api/2.1/unity-catalog/delta/preview/commits",
        get(get_commits).post(post_commit),
    )
}

async fn get_commits(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<DeltaGetCommitsRequest>>,
) -> Result<Json<DeltaGetCommitsResponse>, AppError> {
    authorization::require_allowed(
        authorization::can_vend_table_credentials(&state, &principal, &request.table_id, false)
            .await?,
        "read Delta commits",
    )
    .await?;
    Ok(Json(state.delta_commits.get_commits(request).await?))
}

async fn post_commit(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<DeltaCommitRequest>>,
) -> Result<Json<Value>, AppError> {
    authorization::require_allowed(
        authorization::can_vend_table_credentials(&state, &principal, &request.table_id, true)
            .await?,
        "write Delta commits",
    )
    .await?;
    state.delta_commits.post_commit(request).await?;
    Ok(Json(json!({})))
}
