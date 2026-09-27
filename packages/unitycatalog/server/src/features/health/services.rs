use async_trait::async_trait;
use deadpool_postgres::Pool;

use crate::error::AppError;

use super::dtos::HealthResponse;

#[async_trait]
pub trait HealthService: Send + Sync {
    async fn livez(&self) -> Result<HealthResponse, AppError>;
    async fn readyz(&self) -> Result<HealthResponse, AppError>;
}

pub struct DefaultHealthService {
    pool: Pool,
}

impl DefaultHealthService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthService for DefaultHealthService {
    async fn livez(&self) -> Result<HealthResponse, AppError> {
        Ok(HealthResponse::ok())
    }

    async fn readyz(&self) -> Result<HealthResponse, AppError> {
        let client = self.pool.get().await?;
        // client.simple_query("SELECT 1").await?;
        Ok(HealthResponse::ok())
    }
}
