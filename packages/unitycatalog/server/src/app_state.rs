use std::sync::Arc;

use crate::features::{
    catalogs::CatalogService, health::HealthService, hello::HelloService, schemas::SchemaService,
    vending::VendingService,
};

#[derive(Clone)]
pub struct AppState {
    pub catalogs: Arc<dyn CatalogService>,
    pub health: Arc<dyn HealthService>,
    pub hello: Arc<dyn HelloService>,
    pub schemas: Arc<dyn SchemaService>,
    pub vending: Arc<dyn VendingService>,
}
