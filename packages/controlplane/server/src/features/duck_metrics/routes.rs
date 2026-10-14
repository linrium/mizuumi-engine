use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};

use crate::{app_state::AppState, error::AppError};

use super::dtos::{DuckMetricsResponse, GetDuckMetricsQuery};

const DUCK_METRICS_PATH: &str = "/api/duck-server/metrics";

pub fn duck_metrics_router() -> Router<AppState> {
    Router::new().route(DUCK_METRICS_PATH, get(get_duck_metrics))
}

async fn get_duck_metrics(
    State(state): State<AppState>,
    Query(query): Query<GetDuckMetricsQuery>,
) -> Result<Json<DuckMetricsResponse>, AppError> {
    Ok(Json(state.duck_metrics.query(query, None).await?))
}
