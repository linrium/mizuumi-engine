use axum::{Json, Router, extract::Query, extract::State, routing::get};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{HelloRequest, HelloResponse};

pub fn hello_router() -> Router<AppState> {
    Router::new().route("/api/hello", get(hello))
}

async fn hello(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<HelloRequest>>,
) -> Result<Json<HelloResponse>, AppError> {
    Ok(Json(state.hello.hello(request).await?))
}
