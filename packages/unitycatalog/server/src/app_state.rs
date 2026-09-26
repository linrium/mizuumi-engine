use std::sync::Arc;

use crate::features::{
    catalogs::CatalogService, credentials::CredentialService,
    external_locations::ExternalLocationService, grants::GrantService, health::HealthService,
    hello::HelloService, schemas::SchemaService, tables::TableService, vending::VendingService,
};

#[derive(Clone)]
pub struct AppState {
    pub catalogs: Arc<dyn CatalogService>,
    pub credentials: Arc<dyn CredentialService>,
    pub external_locations: Arc<dyn ExternalLocationService>,
    pub grants: Arc<dyn GrantService>,
    pub health: Arc<dyn HealthService>,
    pub hello: Arc<dyn HelloService>,
    pub schemas: Arc<dyn SchemaService>,
    pub tables: Arc<dyn TableService>,
    pub vending: Arc<dyn VendingService>,
}
