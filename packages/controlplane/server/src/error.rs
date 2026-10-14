use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("authentication provider is unavailable")]
    AuthProviderUnavailable,
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
    #[error("data quality execution failed: {0}")]
    DataQuality(String),
    #[error("SigNoz metrics API is not configured")]
    SignozNotConfigured,
    #[error("SigNoz metrics query failed: {0}")]
    Signoz(String),
    #[error("blast-radius evaluation failed: {0}")]
    BlastRadius(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthProviderUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Postgres(_) | AppError::Pool(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::SignozNotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Kube(kube::Error::Api(resp)) if resp.code == 404 => StatusCode::NOT_FOUND,
            AppError::Kube(_)
            | AppError::UnityCatalog(_)
            | AppError::DataQuality(_)
            | AppError::Signoz(_)
            | AppError::BlastRadius(_) => StatusCode::BAD_GATEWAY,
        };
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        } else {
            tracing::warn!(error = %self, "request rejected");
        }
        let mut response = (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response();
        if status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"controlplane\""),
            );
        }
        response
    }
}
