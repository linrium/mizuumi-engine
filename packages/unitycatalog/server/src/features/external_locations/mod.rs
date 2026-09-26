mod dtos;
mod models;
mod routes;
mod services;

pub use routes::external_location_router;
pub(crate) use services::normalize_external_location_url;
pub use services::{DefaultExternalLocationService, ExternalLocationService};
