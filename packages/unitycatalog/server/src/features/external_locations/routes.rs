use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CreateExternalLocationRequest, DeleteExternalLocationRequest, ExternalLocationInfo,
    ListExternalLocationsRequest, ListExternalLocationsResponse, UpdateExternalLocationRequest,
};

const EXTERNAL_LOCATIONS_PATH: &str = "/api/2.1/unity-catalog/external-locations";

pub fn external_location_router() -> Router<AppState> {
    Router::new()
        .route(
            EXTERNAL_LOCATIONS_PATH,
            post(create_external_location).get(list_external_locations),
        )
        .route(
            &format!("{EXTERNAL_LOCATIONS_PATH}/{{name}}"),
            get(get_external_location)
                .patch(update_external_location)
                .delete(delete_external_location),
        )
}

async fn create_external_location(
    State(state): State<AppState>,
    Valid(Json(request)): Valid<Json<CreateExternalLocationRequest>>,
) -> Result<Json<ExternalLocationInfo>, AppError> {
    Ok(Json(
        state
            .external_locations
            .create_external_location(request)
            .await?,
    ))
}

async fn list_external_locations(
    State(state): State<AppState>,
    Valid(Query(request)): Valid<Query<ListExternalLocationsRequest>>,
) -> Result<Json<ListExternalLocationsResponse>, AppError> {
    Ok(Json(
        state
            .external_locations
            .list_external_locations(request)
            .await?,
    ))
}

async fn get_external_location(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ExternalLocationInfo>, AppError> {
    Ok(Json(
        state.external_locations.get_external_location(name).await?,
    ))
}

async fn update_external_location(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Json(request)): Valid<Json<UpdateExternalLocationRequest>>,
) -> Result<Json<ExternalLocationInfo>, AppError> {
    Ok(Json(
        state
            .external_locations
            .update_external_location(name, request)
            .await?,
    ))
}

async fn delete_external_location(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Valid(Query(request)): Valid<Query<DeleteExternalLocationRequest>>,
) -> Result<StatusCode, AppError> {
    state
        .external_locations
        .delete_external_location(name, request)
        .await?;
    Ok(StatusCode::OK)
}
