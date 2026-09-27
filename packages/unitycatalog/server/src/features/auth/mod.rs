pub mod resource_authorization;
mod routes;
mod service;

pub use routes::auth_router;
pub use service::{AuthService, AuthenticatedPrincipal, authorize};
