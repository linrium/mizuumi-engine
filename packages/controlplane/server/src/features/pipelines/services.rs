use async_trait::async_trait;
use kube::{
    Client,
    api::{Api, DynamicObject, GroupVersionKind, ListParams},
    discovery::ApiResource,
};
use serde_json::Value;

use crate::error::AppError;

use super::dtos::{ExecutorStateSummary, ListPipelinesResponse, Pipeline, PipelineDetail};

const GROUP: &str = "sparkoperator.k8s.io";
const VERSION: &str = "v1beta2";
const KIND: &str = "SparkApplication";

#[async_trait]
pub trait PipelineService: Send + Sync {
    async fn list_pipelines(
        &self,
        namespace: Option<&str>,
    ) -> Result<ListPipelinesResponse, AppError>;
    async fn get_pipeline(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<PipelineDetail, AppError>;
}

pub struct DefaultPipelineService {
    client: Client,
    api_resource: ApiResource,
}

impl DefaultPipelineService {
    pub fn new(client: Client) -> Self {
        let gvk = GroupVersionKind::gvk(GROUP, VERSION, KIND);
        let api_resource = ApiResource::from_gvk(&gvk);
        Self {
            client,
            api_resource,
        }
    }

    fn api(&self, namespace: Option<&str>) -> Api<DynamicObject> {
        match namespace {
            Some(ns) => Api::namespaced_with(self.client.clone(), ns, &self.api_resource),
            None => Api::all_with(self.client.clone(), &self.api_resource),
        }
    }
}

#[async_trait]
impl PipelineService for DefaultPipelineService {
    async fn list_pipelines(
        &self,
        namespace: Option<&str>,
    ) -> Result<ListPipelinesResponse, AppError> {
        let api = self.api(namespace);
        let list = api.list(&ListParams::default()).await?;
        let items = list.items.into_iter().map(summarize).collect();
        Ok(ListPipelinesResponse { items })
    }

    async fn get_pipeline(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<PipelineDetail, AppError> {
        let api = self.api(Some(namespace));
        let obj = api.get(name).await?;
        let raw = serde_json::to_value(&obj).unwrap_or(Value::Null);
        Ok(PipelineDetail {
            pipeline: summarize(obj),
            raw,
        })
    }
}

fn summarize(obj: DynamicObject) -> Pipeline {
    let name = obj.metadata.name.clone().unwrap_or_default();
    let namespace = obj.metadata.namespace.clone().unwrap_or_default();
    let creation_timestamp = obj
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|t| t.0.to_rfc3339());

    let spec = obj.data.get("spec");
    let status = obj.data.get("status");

    let spark_version = spec
        .and_then(|v| v.get("sparkVersion"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let application_type = spec
        .and_then(|v| v.get("type"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mode = spec
        .and_then(|v| v.get("mode"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let image = spec
        .and_then(|v| v.get("image"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let main_application_file = spec
        .and_then(|v| v.get("mainApplicationFile"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let executor_instances = spec
        .and_then(|v| v.get("executor"))
        .and_then(|e| e.get("instances"))
        .and_then(Value::as_i64);

    let state = status
        .and_then(|v| v.get("applicationState"))
        .and_then(|s| s.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_owned();
    let application_state_message = status
        .and_then(|v| v.get("applicationState"))
        .and_then(|s| s.get("errorMessage"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let submission_attempts = status
        .and_then(|v| v.get("submissionAttempts"))
        .and_then(Value::as_i64);
    let execution_attempts = status
        .and_then(|v| v.get("executionAttempts"))
        .and_then(Value::as_i64);
    let submission_time = status
        .and_then(|v| v.get("submissionTime"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let termination_time = status
        .and_then(|v| v.get("terminationTime"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let last_submission_attempt_time = status
        .and_then(|v| v.get("lastSubmissionAttemptTime"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let driver_pod_name = status
        .and_then(|v| v.get("driverInfo"))
        .and_then(|d| d.get("podName"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    let mut executor_state_summary = ExecutorStateSummary::default();
    if let Some(map) = status
        .and_then(|v| v.get("executorState"))
        .and_then(Value::as_object)
    {
        for value in map.values() {
            match value.as_str().unwrap_or("UNKNOWN") {
                "RUNNING" => executor_state_summary.running += 1,
                "COMPLETED" => executor_state_summary.completed += 1,
                "FAILED" => executor_state_summary.failed += 1,
                "PENDING" => executor_state_summary.pending += 1,
                _ => executor_state_summary.unknown += 1,
            }
        }
    }

    Pipeline {
        name,
        namespace,
        state,
        application_state_message,
        spark_version,
        application_type,
        mode,
        image,
        main_application_file,
        submission_attempts,
        execution_attempts,
        submission_time,
        termination_time,
        last_submission_attempt_time,
        creation_timestamp,
        driver_pod_name,
        executor_instances,
        executor_state_summary,
    }
}
