use async_trait::async_trait;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client, api::ListParams};
use serde_json::Value;

use crate::{config::ComputeSettings, error::AppError};

use super::dtos::{Compute, ComputeDetail, ListComputesResponse};

#[async_trait]
pub trait ComputeService: Send + Sync {
    async fn list_computes(&self) -> Result<ListComputesResponse, AppError>;
    async fn get_compute(&self, compute_id: &str) -> Result<ComputeDetail, AppError>;
}

pub struct DefaultComputeService {
    pods: Api<Pod>,
    label_selector: String,
}

impl DefaultComputeService {
    pub fn new(client: Client, settings: &ComputeSettings) -> Self {
        Self {
            pods: Api::namespaced(client, &settings.namespace),
            label_selector: settings.label_selector.clone(),
        }
    }

    async fn matching_pods(&self) -> Result<Vec<Pod>, AppError> {
        Ok(self
            .pods
            .list(&ListParams::default().labels(&self.label_selector))
            .await?
            .items)
    }
}

#[async_trait]
impl ComputeService for DefaultComputeService {
    async fn list_computes(&self) -> Result<ListComputesResponse, AppError> {
        let mut items = self
            .matching_pods()
            .await?
            .into_iter()
            .map(summarize)
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(ListComputesResponse { items })
    }

    async fn get_compute(&self, compute_id: &str) -> Result<ComputeDetail, AppError> {
        let pod = self
            .matching_pods()
            .await?
            .into_iter()
            .find(|pod| pod.metadata.uid.as_deref() == Some(compute_id))
            .ok_or_else(|| AppError::NotFound(format!("compute {compute_id}")))?;
        let raw = serde_json::to_value(&pod).unwrap_or(Value::Null);
        Ok(ComputeDetail {
            compute: summarize(pod),
            raw,
        })
    }
}

fn summarize(pod: Pod) -> Compute {
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

        let compute = summarize(pod);
        assert_eq!(compute.id, "compute-uid");
        assert_eq!(compute.phase, "Running");
        assert!(compute.ready);
        assert_eq!(compute.node_name.as_deref(), Some("worker-1"));
        assert_eq!(compute.restart_count, 2);
    }
}
