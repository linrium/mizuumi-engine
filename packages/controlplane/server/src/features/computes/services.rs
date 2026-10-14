use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api, Client,
    api::{DynamicObject, GroupVersionKind, ListParams},
    discovery::ApiResource,
};
use serde_json::Value;

use crate::{config::ComputeSettings, error::AppError};

use super::dtos::{Compute, ComputeDetail, ComputeKind, ListComputesResponse};

const SPARK_GROUP: &str = "sparkoperator.k8s.io";
const SPARK_VERSION: &str = "v1beta2";
const SPARK_KIND: &str = "SparkApplication";

#[async_trait]
pub trait ComputeService: Send + Sync {
    async fn list_computes(&self) -> Result<ListComputesResponse, AppError>;
    async fn get_compute(&self, compute_id: &str) -> Result<ComputeDetail, AppError>;
}

pub struct DefaultComputeService {
    pods: Api<Pod>,
    label_selector: String,
    spark_applications: Api<DynamicObject>,
}

impl DefaultComputeService {
    pub fn new(client: Client, settings: &ComputeSettings) -> Self {
        let spark_resource = ApiResource::from_gvk(&GroupVersionKind::gvk(
            SPARK_GROUP,
            SPARK_VERSION,
            SPARK_KIND,
        ));
        Self {
            pods: Api::namespaced(client.clone(), &settings.namespace),
            label_selector: settings.label_selector.clone(),
            spark_applications: Api::all_with(client, &spark_resource),
        }
    }

    async fn matching_pods(&self) -> Result<Vec<Pod>, AppError> {
        Ok(self
            .pods
            .list(&ListParams::default().labels(&self.label_selector))
            .await?
            .items)
    }

    async fn spark_apps(&self) -> Result<Vec<DynamicObject>, AppError> {
        Ok(self
            .spark_applications
            .list(&ListParams::default())
            .await?
            .items)
    }
}

#[async_trait]
impl ComputeService for DefaultComputeService {
    async fn list_computes(&self) -> Result<ListComputesResponse, AppError> {
        let (pods, spark_apps) = tokio::try_join!(self.matching_pods(), self.spark_apps())?;
        let mut items = pods
            .into_iter()
            .map(summarize_pod)
            .chain(spark_apps.into_iter().map(summarize_spark_application))
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.kind
                .as_str()
                .cmp(right.kind.as_str())
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(ListComputesResponse { items })
    }

    async fn get_compute(&self, compute_id: &str) -> Result<ComputeDetail, AppError> {
        let (pods, spark_apps) = tokio::try_join!(self.matching_pods(), self.spark_apps())?;
        if let Some(pod) = pods
            .into_iter()
            .find(|pod| pod.metadata.uid.as_deref() == Some(compute_id))
        {
            let raw = serde_json::to_value(&pod).unwrap_or(Value::Null);
            return Ok(ComputeDetail {
                compute: summarize_pod(pod),
                raw,
            });
        }
        if let Some(application) = spark_apps
            .into_iter()
            .find(|application| application.metadata.uid.as_deref() == Some(compute_id))
        {
            let raw = serde_json::to_value(&application).unwrap_or(Value::Null);
            return Ok(ComputeDetail {
                compute: summarize_spark_application(application),
                raw,
            });
        }
        Err(AppError::NotFound(format!("compute {compute_id}")))
    }
}

impl ComputeKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::DuckServer => "duck_server",
            Self::SparkApplication => "spark_application",
        }
    }
}

fn summarize_pod(pod: Pod) -> Compute {
    let status = pod.status.as_ref();
    let container_status = status
        .and_then(|value| value.container_statuses.as_ref())
        .and_then(|values| values.iter().find(|value| value.name == "duck-server"))
        .or_else(|| {
            status
                .and_then(|value| value.container_statuses.as_ref())
                .and_then(|values| values.first())
        });
    let ready = status
        .and_then(|value| value.conditions.as_ref())
        .and_then(|values| values.iter().find(|value| value.type_ == "Ready"))
        .is_some_and(|condition| condition.status == "True");

    Compute {
        id: pod.metadata.uid.clone().unwrap_or_default(),
        kind: ComputeKind::DuckServer,
        name: pod.metadata.name.clone().unwrap_or_default(),
        namespace: pod.metadata.namespace.clone().unwrap_or_default(),
        phase: status
            .and_then(|value| value.phase.clone())
            .unwrap_or_else(|| "Unknown".to_owned()),
        ready,
        node_name: pod.spec.as_ref().and_then(|value| value.node_name.clone()),
        pod_ip: status.and_then(|value| value.pod_ip.clone()),
        image: pod
            .spec
            .as_ref()
            .and_then(|value| value.containers.first())
            .and_then(|value| value.image.clone()),
        image_id: container_status.map(|value| value.image_id.clone()),
        restart_count: container_status.map_or(0, |value| value.restart_count),
        started_at: status
            .and_then(|value| value.start_time.as_ref())
            .map(|value| value.0.to_rfc3339()),
        creation_timestamp: pod
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|value| value.0.to_rfc3339()),
        spark_version: None,
        driver_pod_name: None,
        executor_instances: None,
    }
}

fn summarize_spark_application(application: DynamicObject) -> Compute {
    let spec = application.data.get("spec");
    let status = application.data.get("status");
    let state = status
        .and_then(|value| value.get("applicationState"))
        .and_then(|value| value.get("state"))
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_owned();

    Compute {
        id: application.metadata.uid.clone().unwrap_or_default(),
        kind: ComputeKind::SparkApplication,
        name: application.metadata.name.clone().unwrap_or_default(),
        namespace: application.metadata.namespace.clone().unwrap_or_default(),
        ready: state == "RUNNING",
        phase: state,
        node_name: None,
        pod_ip: None,
        image: spec
            .and_then(|value| value.get("image"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        image_id: None,
        restart_count: 0,
        started_at: status
            .and_then(|value| value.get("submissionTime"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        creation_timestamp: application
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|value| value.0.to_rfc3339()),
        spark_version: spec
            .and_then(|value| value.get("sparkVersion"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        driver_pod_name: status
            .and_then(|value| value.get("driverInfo"))
            .and_then(|value| value.get("podName"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        executor_instances: spec
            .and_then(|value| value.get("executor"))
            .and_then(|value| value.get("instances"))
            .and_then(Value::as_i64),
    }
}

#[cfg(test)]
mod tests {
    use k8s_openapi::{
        api::core::v1::{Container, ContainerStatus, PodCondition, PodSpec, PodStatus},
        apimachinery::pkg::apis::meta::v1::ObjectMeta,
    };

    use super::*;

    #[test]
    fn summarizes_a_duck_server_pod() {
        let pod = Pod {
            metadata: ObjectMeta {
                name: Some("duck-server-7f9d".to_owned()),
                namespace: Some("duck-server".to_owned()),
                uid: Some("compute-uid".to_owned()),
                ..Default::default()
            },
            spec: Some(PodSpec {
                node_name: Some("worker-1".to_owned()),
                containers: vec![Container {
                    name: "duck-server".to_owned(),
                    image: Some("mizuumi/duck-server:latest".to_owned()),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            status: Some(PodStatus {
                phase: Some("Running".to_owned()),
                pod_ip: Some("10.0.0.8".to_owned()),
                conditions: Some(vec![PodCondition {
                    type_: "Ready".to_owned(),
                    status: "True".to_owned(),
                    ..Default::default()
                }]),
                container_statuses: Some(vec![ContainerStatus {
                    name: "duck-server".to_owned(),
                    image: "mizuumi/duck-server:latest".to_owned(),
                    image_id: "sha256:image".to_owned(),
                    ready: true,
                    restart_count: 2,
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        };

        let compute = summarize_pod(pod);
        assert_eq!(compute.id, "compute-uid");
        assert_eq!(compute.kind, ComputeKind::DuckServer);
        assert_eq!(compute.phase, "Running");
        assert!(compute.ready);
        assert_eq!(compute.node_name.as_deref(), Some("worker-1"));
        assert_eq!(compute.restart_count, 2);
    }

    #[test]
    fn summarizes_a_spark_application() {
        let application: DynamicObject = serde_json::from_value(serde_json::json!({
            "apiVersion": "sparkoperator.k8s.io/v1beta2",
            "kind": "SparkApplication",
            "metadata": {
                "name": "crm-bronze-to-silver",
                "namespace": "spark",
                "uid": "spark-uid"
            },
            "spec": {
                "image": "mizuumi/spark:latest",
                "sparkVersion": "4.0.1",
                "executor": {"instances": 2}
            },
            "status": {
                "applicationState": {"state": "RUNNING"},
                "submissionTime": "2026-10-14T10:00:00Z",
                "driverInfo": {"podName": "crm-bronze-to-silver-driver"}
            }
        }))
        .unwrap();

        let compute = summarize_spark_application(application);
        assert_eq!(compute.id, "spark-uid");
        assert_eq!(compute.kind, ComputeKind::SparkApplication);
        assert_eq!(compute.phase, "RUNNING");
        assert!(compute.ready);
        assert_eq!(compute.spark_version.as_deref(), Some("4.0.1"));
        assert_eq!(compute.executor_instances, Some(2));
    }
}
