use axum::{Extension, Json, Router, routing::get};

use crate::app_state::AppState;

use super::AuthenticatedPrincipal;

pub fn auth_router() -> Router<AppState> {
    Router::new().route("/api/auth/me", get(current_principal))
}

async fn current_principal(
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> Json<AuthenticatedPrincipal> {
    Json(principal)
}
