use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Compute {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub ready: bool,
    pub node_name: Option<String>,
    pub pod_ip: Option<String>,
    pub image: Option<String>,
    pub image_id: Option<String>,
    pub restart_count: i32,
    pub started_at: Option<String>,
    pub creation_timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComputeDetail {
    #[serde(flatten)]
    pub compute: Compute,
    pub raw: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ListComputesResponse {
    pub items: Vec<Compute>,
}
