use axum::{Json, Router, extract::State, routing::post};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    GenerateTemporaryModelVersionCredentialRequest, GenerateTemporaryPathCredentialRequest,
    GenerateTemporaryTableCredentialRequest, GenerateTemporaryVolumeCredentialRequest,
    TemporaryCredentials,
};

pub fn temporary_credentials_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/2.1/unity-catalog/temporary-model-version-credentials",
            post(generate_temporary_model_version_credentials),
        )
        .route(
            "/api/2.1/unity-catalog/temporary-path-credentials",
            post(generate_temporary_path_credentials),
        )
        .route(
            "/api/2.1/unity-catalog/temporary-table-credentials",
            post(generate_temporary_table_credentials),
        )
        .route(
            "/api/2.1/unity-catalog/temporary-volume-credentials",
            post(generate_temporary_volume_credentials),
        )
}

async fn generate_temporary_model_version_credentials(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryModelVersionCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    Ok(Json(
        state
            .temporary_credentials
            .generate_model_version_credentials(request)
            .await?,
    ))
}

async fn generate_temporary_path_credentials(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryPathCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    Ok(Json(
        state
            .temporary_credentials
            .generate_path_credentials(request)
            .await?,
    ))
}

async fn generate_temporary_table_credentials(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryTableCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    Ok(Json(
        state
            .temporary_credentials
            .generate_table_credentials(request)
            .await?,
    ))
}

async fn generate_temporary_volume_credentials(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryVolumeCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    Ok(Json(
        state
            .temporary_credentials
            .generate_volume_credentials(request)
            .await?,
    ))
}
