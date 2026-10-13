use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct PutDataContractRequest {
    pub document: Value,
    pub created_by: String,
}

#[derive(Debug, Serialize)]
pub struct DataContract {
    pub id: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_name: String,
    pub version: String,
    pub document: Value,
    pub document_hash: String,
    pub schema_hash: String,
    pub created_by: String,
    pub created_at: String,
    pub sync_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unity_table_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_schema_hash: Option<String>,
    pub differences: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunDataQualityRequest {
    pub triggered_by: String,
}

#[derive(Debug, Serialize)]
pub struct DataQualityRuns {
    pub quality_runs: Vec<DataQualityRun>,
}

#[derive(Debug, Serialize)]
pub struct DataQualityRun {
    pub id: String,
    pub contract_id: String,
    pub contract_version: String,
    pub engine: String,
    pub status: String,
    pub trigger_type: String,
    pub triggered_by: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub passed_count: i32,
    pub failed_count: i32,
    pub error_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub results: Vec<DataQualityResult>,
}

#[derive(Debug, Serialize)]
pub struct DataQualityResult {
    pub rule_id: String,
    pub rule_scope: String,
    pub rule_type: String,
    pub metric: String,
    pub severity: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric_value: Option<Value>,
    pub expectation: Value,
    pub duration_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
