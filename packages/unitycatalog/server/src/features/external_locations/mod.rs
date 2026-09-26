mod dtos;
mod models;
mod routes;
mod services;

pub use routes::external_location_router;
pub use services::{DefaultExternalLocationService, ExternalLocationService};
