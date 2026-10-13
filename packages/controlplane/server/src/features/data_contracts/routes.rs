use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, header},
    routing::{get, post, put},
};

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    DataContract, DataQualityRun, DataQualityRuns, PutDataContractRequest, RunDataQualityRequest,
};

const DATA_CONTRACTS_PATH: &str = "/api/data-contracts";

pub fn data_contract_router() -> Router<AppState> {
    Router::new()
        .route(
            &format!("{DATA_CONTRACTS_PATH}/{{full_name}}"),
            get(get_data_contract),
        )
        .route(
            &format!("{DATA_CONTRACTS_PATH}/{{full_name}}/versions/{{version}}"),
            put(put_data_contract),
        )
        .route(
            &format!("{DATA_CONTRACTS_PATH}/{{full_name}}/reconcile"),
            post(reconcile_data_contract),
        )
        .route(
            &format!("{DATA_CONTRACTS_PATH}/{{full_name}}/quality-runs"),
            get(list_data_quality_runs).post(run_data_quality),
        )
}

async fn put_data_contract(
    State(state): State<AppState>,
    Path((full_name, version)): Path<(String, String)>,
    Json(request): Json<PutDataContractRequest>,
) -> Result<Json<DataContract>, AppError> {
    Ok(Json(
        state
            .data_contracts
            .put_data_contract(full_name, version, request)
            .await?,
    ))
}

async fn get_data_contract(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
) -> Result<Json<DataContract>, AppError> {
    Ok(Json(
        state.data_contracts.get_data_contract(full_name).await?,
    ))
}

async fn reconcile_data_contract(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
) -> Result<Json<DataContract>, AppError> {
    Ok(Json(
        state
            .data_contracts
            .reconcile_data_contract(full_name)
            .await?,
    ))
}

async fn list_data_quality_runs(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
) -> Result<Json<DataQualityRuns>, AppError> {
    Ok(Json(
        state
            .data_contracts
            .list_data_quality_runs(full_name)
            .await?,
    ))
}

async fn run_data_quality(
    State(state): State<AppState>,
    Path(full_name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<RunDataQualityRequest>,
) -> Result<Json<DataQualityRun>, AppError> {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.starts_with("Bearer "))
        .ok_or_else(|| AppError::Forbidden("bearer token required".to_owned()))?;
    Ok(Json(
        state
            .data_contracts
            .run_data_quality(full_name, authorization.to_owned(), request)
            .await?,
    ))
}
