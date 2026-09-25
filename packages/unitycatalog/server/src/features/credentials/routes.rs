use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CreateCredentialRequest, CredentialInfo, DeleteCredentialRequest, ListCredentialsRequest,
    ListCredentialsResponse, UpdateCredentialRequest,
};

const CREDENTIALS_PATH: &str = "/api/2.1/unity-catalog/credentials";

pub fn credential_router() -> Router<AppState> {
    Router::new()
        .route(
            CREDENTIALS_PATH,
            post(create_credential).get(list_credentials),
        )
        .route(
            &format!("{CREDENTIALS_PATH}/{{name}}"),
            get(get_credential)
                .patch(update_credential)
                .delete(delete_credential),
        )
}

async fn create_credential(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<CreateCredentialRequest>>,
) -> Result<Json<CredentialInfo>, AppError> {
    Ok(Json(state.credentials.create_credential(request).await?))
}

async fn list_credentials(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListCredentialsRequest>>,
) -> Result<Json<ListCredentialsResponse>, AppError> {
    Ok(Json(state.credentials.list_credentials(request).await?))
}

async fn get_credential(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CredentialInfo>, AppError> {
    Ok(Json(state.credentials.get_credential(name).await?))
}

async fn update_credential(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Json(request)): Valid<Json<UpdateCredentialRequest>>,
) -> Result<Json<CredentialInfo>, AppError> {
    Ok(Json(
        state.credentials.update_credential(name, request).await?,
    ))
}

async fn delete_credential(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Query(request)): Valid<Query<DeleteCredentialRequest>>,
) -> Result<StatusCode, AppError> {
    state.credentials.delete_credential(name, request).await?;
    Ok(StatusCode::OK)
}
