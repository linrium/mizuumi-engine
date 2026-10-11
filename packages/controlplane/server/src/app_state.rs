use std::sync::Arc;

use crate::features::health::HealthService;

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<dyn HealthService>,
}
