use std::sync::Arc;

use crate::features::{
    catalogs::CatalogService, credentials::CredentialService, health::HealthService,
    hello::HelloService, schemas::SchemaService, tables::TableService, vending::VendingService,
};

#[derive(Clone)]
pub struct AppState {
    pub catalogs: Arc<dyn CatalogService>,
    pub credentials: Arc<dyn CredentialService>,
    pub health: Arc<dyn HealthService>,
    pub hello: Arc<dyn HelloService>,
    pub schemas: Arc<dyn SchemaService>,
    pub tables: Arc<dyn TableService>,
    pub vending: Arc<dyn VendingService>,
}
