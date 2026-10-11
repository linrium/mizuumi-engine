use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{ListPipelinesResponse, PipelineDetail};

const PIPELINES_PATH: &str = "/api/pipelines";

#[derive(Debug, Deserialize)]
pub struct ListPipelinesQuery {
    pub namespace: Option<String>,
}

pub fn pipeline_router() -> Router<AppState> {
    Router::new()
        .route(PIPELINES_PATH, get(list_pipelines))
        .route(
            &format!("{PIPELINES_PATH}/{{namespace}}/{{name}}"),
            get(get_pipeline),
        )
}

async fn list_pipelines(
    State(state): State<AppState>,
    Query(query): Query<ListPipelinesQuery>,
) -> Result<Json<ListPipelinesResponse>, AppError> {
    Ok(Json(
        state
            .pipelines
            .list_pipelines(query.namespace.as_deref())
            .await?,
    ))
}

async fn get_pipeline(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<PipelineDetail>, AppError> {
    Ok(Json(state.pipelines.get_pipeline(&namespace, &name).await?))
}
