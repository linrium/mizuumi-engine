use std::sync::Arc;

use crate::features::{
    access_requests::AccessRequestService, health::HealthService,
    policy_templates::PolicyTemplateService,
};

#[derive(Clone)]
pub struct AppState {
    pub access_requests: Arc<dyn AccessRequestService>,
    pub health: Arc<dyn HealthService>,
    pub policy_templates: Arc<dyn PolicyTemplateService>,
}
