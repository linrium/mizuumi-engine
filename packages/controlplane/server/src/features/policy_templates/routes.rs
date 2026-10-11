use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};

use crate::{app_state::AppState, error::AppError};

use super::dtos::{
    CreatePolicyTemplateRequest, ListPolicyTemplatesRequest, ListPolicyTemplatesResponse,
    PolicyTemplate,
};

const POLICY_TEMPLATES_PATH: &str = "/api/policy-templates";

pub fn policy_template_router() -> Router<AppState> {
    Router::new()
        .route(
            POLICY_TEMPLATES_PATH,
            post(create_policy_template).get(list_policy_templates),
        )
        .route(
            &format!("{POLICY_TEMPLATES_PATH}/{{template_id}}"),
            get(get_policy_template),
        )
}

async fn create_policy_template(
    State(state): State<AppState>,
    Json(request): Json<CreatePolicyTemplateRequest>,
) -> Result<(StatusCode, Json<PolicyTemplate>), AppError> {
    let template = state
        .policy_templates
        .create_policy_template(request)
        .await?;
    Ok((StatusCode::CREATED, Json(template)))
}

async fn list_policy_templates(
    State(state): State<AppState>,
    Query(request): Query<ListPolicyTemplatesRequest>,
) -> Result<Json<ListPolicyTemplatesResponse>, AppError> {
    Ok(Json(
        state
            .policy_templates
            .list_policy_templates(request)
            .await?,
    ))
}

async fn get_policy_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<PolicyTemplate>, AppError> {
    Ok(Json(
        state
            .policy_templates
            .get_policy_template(template_id)
            .await?,
    ))
}
