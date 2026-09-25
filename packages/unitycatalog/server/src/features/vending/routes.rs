use axum::{Json, Router, extract::Query, extract::State, routing::get};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{ListBucketsRequest, ListBucketsResponse};

pub fn vending_router() -> Router<AppState> {
    Router::new().route("/api/vending/buckets", get(list_buckets))
}

async fn list_buckets(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListBucketsRequest>>,
) -> Result<Json<ListBucketsResponse>, AppError> {
    Ok(Json(state.vending.list_buckets(request).await?))
}
