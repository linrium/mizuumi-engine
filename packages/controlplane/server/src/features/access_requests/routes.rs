use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    AccessRequest, CreateAccessRequest, ListAccessRequestsRequest, ListAccessRequestsResponse,
    ReviewAccessRequest,
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
            get(get_access_request),
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
            "/api/approval-teams/{approval_team}/access-requests",
            get(list_approval_team_requests),
        )
}

async fn create_access_request(
    State(state): State<AppState>,
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

async fn approve_access_request(
    State(state): State<AppState>,
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

async fn reject_access_request(
    State(state): State<AppState>,
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
