use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("postgres query failed")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("postgres pool failed")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("JSON serialization failed")]
    Json(#[from] serde_json::Error),
    #[error("kubernetes api failed: {0}")]
    Kube(#[from] kube::Error),
    #[error("Unity Catalog synchronization failed: {0}")]
    UnityCatalog(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Postgres(_) | AppError::Pool(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Kube(kube::Error::Api(resp)) if resp.code == 404 => StatusCode::NOT_FOUND,
            AppError::Kube(_) | AppError::UnityCatalog(_) => StatusCode::BAD_GATEWAY,
        };
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        } else {
            tracing::warn!(error = %self, "request rejected");
        }
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
