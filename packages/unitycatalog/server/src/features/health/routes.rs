use axum::{Json, Router, extract::State, routing::get};

use crate::{app_state::AppState, error::AppError};

use super::dtos::HealthResponse;

pub fn health_router() -> Router<AppState> {
    Router::new()
        .route("/health/readyz", get(readyz))
        .route("/health/livez", get(livez))
        .route("/heath/livez", get(livez))
}

async fn livez(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(state.health.livez().await?))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    Ok(Json(state.health.readyz().await?))
}
