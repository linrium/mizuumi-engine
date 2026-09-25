use std::sync::Arc;

use crate::features::{
    catalogs::CatalogService, health::HealthService, hello::HelloService, vending::VendingService,
};

#[derive(Clone)]
pub struct AppState {
    pub catalogs: Arc<dyn CatalogService>,
    pub health: Arc<dyn HealthService>,
    pub hello: Arc<dyn HelloService>,
    pub vending: Arc<dyn VendingService>,
}
