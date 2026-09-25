use async_trait::async_trait;
use deadpool_postgres::Pool;
use unitycatalog_queries::queries::hello::hello_message;

use crate::error::AppError;

use super::{
    dtos::{HelloRequest, HelloResponse},
    models::HelloMessage,
};

#[async_trait]
pub trait HelloService: Send + Sync {
    async fn hello(&self, request: HelloRequest) -> Result<HelloResponse, AppError>;
}

pub struct DefaultHelloService {
    pool: Pool,
}

impl DefaultHelloService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HelloService for DefaultHelloService {
    async fn hello(&self, request: HelloRequest) -> Result<HelloResponse, AppError> {
        let _name = request.name;
        let client = self.pool.get().await?;
        let message = hello_message().bind(&client).one().await?;

        Ok(HelloMessage { value: message }.into())
    }
}
