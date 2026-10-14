use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};

use crate::{app_state::AppState, error::AppError, features::auth::AuthenticatedPrincipal};

use super::dtos::{
    AccessRequest, CancelAccessRequest, CreateAccessRequest, ListAccessRequestsRequest,
    ListAccessRequestsResponse, ReviewAccessRequest, UpdateAccessRequest,
};

const ACCESS_REQUESTS_PATH: &str = "/api/access-requests";

pub fn access_request_router() -> Router<AppState> {
    Router::new()
        .route(
            ACCESS_REQUESTS_PATH,
            post(create_access_request).get(list_access_requests),
        )
        .route(
            &format!("{ACCESS_REQUESTS_PATH}/{{request_id}}"),
            get(get_access_request).patch(update_access_request),
        )
        .route(
            &format!("{ACCESS_REQUESTS_PATH}/{{request_id}}/approve"),
            post(approve_access_request),
        )
        .route(
            &format!("{ACCESS_REQUESTS_PATH}/{{request_id}}/reject"),
            post(reject_access_request),
        )
        .route(
            &format!("{ACCESS_REQUESTS_PATH}/{{request_id}}/cancel"),
            post(cancel_access_request),
        )
        .route(
            "/api/approval-teams/{approval_team}/access-requests",
            get(list_approval_team_requests),
        )
}

#[chronicle_audit::audit(
    action = "access_requests.create",
    principal = principal.id,
    fields(
        requested_by = request.requested_by,
        target_securable_type = request.target_securable_type,
        target_full_name = request.target_full_name,
        requested_privilege = request.requested_privilege,
    ),
)]
async fn create_access_request(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<CreateAccessRequest>,
) -> Result<(StatusCode, Json<AccessRequest>), AppError> {
    let request = state.access_requests.create_access_request(request).await?;
    Ok((StatusCode::CREATED, Json(request)))
}

async fn list_access_requests(
    State(state): State<AppState>,
    Query(request): Query<ListAccessRequestsRequest>,
) -> Result<Json<ListAccessRequestsResponse>, AppError> {
    Ok(Json(
        state.access_requests.list_access_requests(request).await?,
    ))
}

async fn list_approval_team_requests(
    State(state): State<AppState>,
    Path(approval_team): Path<String>,
    Query(mut request): Query<ListAccessRequestsRequest>,
) -> Result<Json<ListAccessRequestsResponse>, AppError> {
    request.approval_team = Some(approval_team);
    Ok(Json(
        state.access_requests.list_access_requests(request).await?,
    ))
}

async fn get_access_request(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
) -> Result<Json<AccessRequest>, AppError> {
    Ok(Json(
        state.access_requests.get_access_request(request_id).await?,
    ))
}

#[chronicle_audit::audit(
    action = "access_requests.update",
    principal = principal.id,
    fields(request_id = request_id, requested_by = request.requested_by),
)]
async fn update_access_request(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(request_id): Path<String>,
    Json(request): Json<UpdateAccessRequest>,
) -> Result<Json<AccessRequest>, AppError> {
    Ok(Json(
        state
            .access_requests
            .update_access_request(request_id, request)
            .await?,
    ))
}

#[chronicle_audit::audit(
    action = "access_requests.cancel",
    principal = principal.id,
    fields(request_id = request_id, requested_by = request.requested_by),
)]
async fn cancel_access_request(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(request_id): Path<String>,
    Json(request): Json<CancelAccessRequest>,
) -> Result<Json<AccessRequest>, AppError> {
    Ok(Json(
        state
            .access_requests
            .cancel_access_request(request_id, request)
            .await?,
    ))
}

#[chronicle_audit::audit(
    action = "access_requests.approve",
    principal = principal.id,
    fields(
        request_id = request_id,
        approval_team = review.approval_team,
        reviewer = review.reviewer,
    ),
)]
async fn approve_access_request(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(request_id): Path<String>,
    Json(review): Json<ReviewAccessRequest>,
) -> Result<Json<AccessRequest>, AppError> {
    Ok(Json(
        state
            .access_requests
            .approve_access_request(request_id, review)
            .await?,
    ))
}

#[chronicle_audit::audit(
    action = "access_requests.reject",
    principal = principal.id,
    fields(
        request_id = request_id,
        approval_team = review.approval_team,
        reviewer = review.reviewer,
    ),
)]
async fn reject_access_request(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(request_id): Path<String>,
    Json(review): Json<ReviewAccessRequest>,
) -> Result<Json<AccessRequest>, AppError> {
    Ok(Json(
        state
            .access_requests
            .reject_access_request(request_id, review)
            .await?,
    ))
}
