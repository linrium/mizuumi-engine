use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::HelloMessage;

#[derive(Debug, Deserialize, Validate)]
pub struct HelloRequest {
    #[validate(length(min = 1, max = 64))]
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct HelloResponse {
    pub message: String,
}

impl From<HelloMessage> for HelloResponse {
    fn from(message: HelloMessage) -> Self {
        Self {
            message: message.value,
        }
    }
}
