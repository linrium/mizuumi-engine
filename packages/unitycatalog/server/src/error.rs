use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    Conflict(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
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
        tracing::error!(error = %self, "request failed");

        let status = match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
            AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Postgres(_)
            | AppError::Pool(_)
            | AppError::Sts
            | AppError::StsCredentials
            | AppError::S3 => StatusCode::SERVICE_UNAVAILABLE,
        };

        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
