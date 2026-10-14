use std::sync::Arc;

use crate::features::{
    access_requests::AccessRequestService, computes::ComputeService,
    data_contracts::DataContractService, duck_metrics::DuckMetricsService, health::HealthService,
    pipelines::PipelineService, policy_templates::PolicyTemplateService,
    spark_metrics::SparkMetricsService,
};

#[derive(Clone)]
pub struct AppState {
    pub access_requests: Arc<dyn AccessRequestService>,
    pub data_contracts: Arc<dyn DataContractService>,
    pub computes: Arc<dyn ComputeService>,
    pub duck_metrics: Arc<dyn DuckMetricsService>,
    pub spark_metrics: Arc<dyn SparkMetricsService>,
    pub health: Arc<dyn HealthService>,
    pub pipelines: Arc<dyn PipelineService>,
    pub policy_templates: Arc<dyn PolicyTemplateService>,
}
