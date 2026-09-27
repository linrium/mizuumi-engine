use axum::{
    Json, Router,
    extract::{Extension, State},
    routing::post,
};
use axum_valid::Valid;

use crate::{
    app_state::AppState,
    error::AppError,
    features::auth::{AuthenticatedPrincipal, resource_authorization as authorization},
};

use super::dtos::{
    GenerateTemporaryModelVersionCredentialRequest, GenerateTemporaryPathCredentialRequest,
    GenerateTemporaryTableCredentialRequest, GenerateTemporaryVolumeCredentialRequest,
    PathOperation, TableOperation, TemporaryCredentials,
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
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryPathCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    let read_write = match request.operation {
        PathOperation::PathRead => false,
        PathOperation::PathReadWrite | PathOperation::PathCreateTable => true,
        PathOperation::UnknownPathOperation => {
            return Err(AppError::InvalidParameter(
                "unknown operation in the request: UNKNOWN_PATH_OPERATION".to_string(),
            ));
        }
    };
    authorization::require_allowed(
        authorization::can_vend_path_credentials(&state, &principal, &request.url, read_write)
            .await?,
        "access external location",
    )
    .await?;
    Ok(Json(
        state
            .temporary_credentials
            .generate_path_credentials(request)
            .await?,
    ))
}

async fn generate_temporary_table_credentials(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Valid(Json(request)): Valid<Json<GenerateTemporaryTableCredentialRequest>>,
) -> Result<Json<TemporaryCredentials>, AppError> {
    let read_write = matches!(request.operation, TableOperation::ReadWrite);
    authorization::require_allowed(
        authorization::can_vend_table_credentials(
            &state,
            &principal,
            &request.table_id,
            read_write,
        )
        .await?,
        "access table storage",
    )
    .await?;
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
