use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("postgres query failed")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("postgres pool failed")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("rustfs sts request failed")]
    StsRequest(#[from] reqwest::Error),
    #[error("rustfs sts request signing failed")]
    StsSigning,
    #[error("rustfs sts returned an invalid response")]
    StsResponse,
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
            AppError::Postgres(_)
            | AppError::Pool(_)
            | AppError::StsRequest(_)
            | AppError::StsSigning
            | AppError::StsResponse
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
