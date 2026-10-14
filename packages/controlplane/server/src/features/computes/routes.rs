use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

use crate::{
    app_state::AppState,
    error::AppError,
    features::computes::ComputeKind,
    features::duck_metrics::{DuckMetricFilter, DuckMetricsResponse, GetDuckMetricsQuery},
    features::spark_metrics::{SparkMetricFilter, SparkMetricsResponse},
};
use serde::Serialize;

use super::dtos::{ComputeDetail, ListComputesResponse};

const COMPUTES_PATH: &str = "/api/computes";

#[derive(Serialize)]
#[serde(untagged)]
enum ComputeMetricsResponse {
    Duck(DuckMetricsResponse),
    Spark(SparkMetricsResponse),
}

pub fn compute_router() -> Router<AppState> {
    Router::new()
        .route(COMPUTES_PATH, get(list_computes))
        .route(&format!("{COMPUTES_PATH}/{{compute_id}}"), get(get_compute))
        .route(
            &format!("{COMPUTES_PATH}/{{compute_id}}/metrics"),
            get(get_compute_metrics),
        )
}

async fn list_computes(
    State(state): State<AppState>,
) -> Result<Json<ListComputesResponse>, AppError> {
    Ok(Json(state.computes.list_computes().await?))
}

async fn get_compute(
    State(state): State<AppState>,
    Path(compute_id): Path<String>,
) -> Result<Json<ComputeDetail>, AppError> {
    Ok(Json(state.computes.get_compute(&compute_id).await?))
}

async fn get_compute_metrics(
    State(state): State<AppState>,
    Path(compute_id): Path<String>,
    Query(query): Query<GetDuckMetricsQuery>,
) -> Result<Json<ComputeMetricsResponse>, AppError> {
    let compute = state.computes.get_compute(&compute_id).await?;
    match compute.compute.kind {
        ComputeKind::DuckServer => {
            let filter = DuckMetricFilter {
                compute_id: compute.compute.id,
            };
            Ok(Json(ComputeMetricsResponse::Duck(
                state.duck_metrics.query(query, Some(&filter)).await?,
            )))
        }
        ComputeKind::SparkApplication => {
            let filter = SparkMetricFilter {
                compute_id: compute.compute.id,
                application_name: compute.compute.name,
            };
            Ok(Json(ComputeMetricsResponse::Spark(
                state.spark_metrics.query(query, &filter).await?,
            )))
        }
    }
}
