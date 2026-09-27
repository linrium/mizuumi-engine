mod dtos;
mod models;
mod routes;
mod services;

pub use routes::external_location_router;
pub use services::{DefaultExternalLocationService, ExternalLocationService};
pub(crate) use services::{normalize_external_location_url, normalize_managed_table_url};
