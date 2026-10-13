use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put},
};

use crate::{app_state::AppState, error::AppError};

use super::dtos::{DataContract, PutDataContractRequest};

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
