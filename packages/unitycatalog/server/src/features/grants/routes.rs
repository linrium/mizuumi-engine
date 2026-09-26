use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use axum_valid::Valid;

use crate::{app_state::AppState, error::AppError};

use super::dtos::{GetPermissionsRequest, PermissionsList, SecurableType, UpdatePermissions};

const PERMISSIONS_PATH: &str = "/api/2.1/unity-catalog/permissions/{securable_type}/{full_name}";

pub fn grant_router() -> Router<AppState> {
    Router::new().route(
        PERMISSIONS_PATH,
        get(get_permissions).patch(update_permissions),
    )
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
