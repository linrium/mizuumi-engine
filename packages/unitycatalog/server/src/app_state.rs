use std::sync::Arc;

use crate::features::{health::HealthService, hello::HelloService, vending::VendingService};

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<dyn HealthService>,
    pub hello: Arc<dyn HelloService>,
    pub vending: Arc<dyn VendingService>,
}
