use serde::Serialize;

use super::models::HealthStatus;

#[derive(Serialize)]
pub struct HealthResponse {
    status: HealthStatus,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: HealthStatus::Ok,
        }
    }
}
