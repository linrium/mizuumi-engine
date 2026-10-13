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
