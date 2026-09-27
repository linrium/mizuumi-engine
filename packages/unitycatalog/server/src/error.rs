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
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("authentication provider is unavailable")]
    AuthProviderUnavailable,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    Conflict(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),
    #[error("json conversion failed")]
    Json(#[from] serde_json::Error),
    #[error("postgres query failed")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("postgres pool failed")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("rustfs sts request failed")]
    Sts,
    #[error("rustfs sts response did not include credentials")]
    StsCredentials,
    #[error("s3 request failed")]
    S3,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match &self {
            AppError::Unauthorized(_) | AppError::Forbidden(_) => {
                tracing::warn!(error = %self, "request rejected")
            }
            _ => tracing::error!(error = %self, "request failed"),
        }

        let status = match &self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::AuthProviderUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
            AppError::FailedPrecondition(_) => StatusCode::BAD_REQUEST,
            AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Postgres(_)
            | AppError::Pool(_)
            | AppError::Sts
            | AppError::StsCredentials
            | AppError::S3 => StatusCode::SERVICE_UNAVAILABLE,
        };

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
                HeaderValue::from_static("Bearer realm=\"unitycatalog\""),
            );
        }
        response
    }
}
