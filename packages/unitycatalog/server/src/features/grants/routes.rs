use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    AllAssignmentsList, GetPermissionsRequest, PermissionsList, SecurableType, UpdatePermissions,
};

const PERMISSIONS_PATH: &str = "/api/2.1/unity-catalog/permissions/{securable_type}/{full_name}";
const ALL_ASSIGNMENTS_PATH: &str = "/api/2.1/unity-catalog/permissions";

pub fn grant_router() -> Router<AppState> {
    Router::new()
        .route(ALL_ASSIGNMENTS_PATH, get(list_all_assignments))
        .route(
            PERMISSIONS_PATH,
            get(get_permissions).patch(update_permissions),
        )
}

async fn list_all_assignments(
    State(state): State<AppState>,
) -> Result<Json<AllAssignmentsList>, AppError> {
    Ok(Json(state.grants.list_all_assignments().await?))
}

async fn get_permissions(
    State(state): State<AppState>,
    Path((securable_type, full_name)): Path<(SecurableType, String)>,
    Valid(Query(request)): Valid<Query<GetPermissionsRequest>>,
) -> Result<Json<PermissionsList>, AppError> {
    Ok(Json(
        state
            .grants
            .get_permissions(securable_type, full_name, request)
            .await?,
    ))
}

async fn update_permissions(
    State(state): State<AppState>,
    Path((securable_type, full_name)): Path<(SecurableType, String)>,
    Valid(Json(request)): Valid<Json<UpdatePermissions>>,
) -> Result<Json<PermissionsList>, AppError> {
    Ok(Json(
        state
            .grants
            .update_permissions(securable_type, full_name, request)
            .await?,
    ))
}
