use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Pipeline {
    pub name: String,
    pub namespace: String,
    pub state: String,
    pub application_state_message: Option<String>,
    pub spark_version: Option<String>,
    pub application_type: Option<String>,
    pub mode: Option<String>,
    pub image: Option<String>,
    pub main_application_file: Option<String>,
    pub submission_attempts: Option<i64>,
    pub execution_attempts: Option<i64>,
    pub submission_time: Option<String>,
    pub termination_time: Option<String>,
    pub last_submission_attempt_time: Option<String>,
    pub creation_timestamp: Option<String>,
    pub driver_pod_name: Option<String>,
    pub executor_instances: Option<i64>,
    pub executor_state_summary: ExecutorStateSummary,
}

#[derive(Debug, Default, Serialize)]
pub struct ExecutorStateSummary {
    pub running: u32,
    pub completed: u32,
    pub failed: u32,
    pub pending: u32,
    pub unknown: u32,
}

#[derive(Debug, Serialize)]
pub struct PipelineDetail {
    #[serde(flatten)]
    pub pipeline: Pipeline,
    pub raw: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ListPipelinesResponse {
    pub items: Vec<Pipeline>,
}
