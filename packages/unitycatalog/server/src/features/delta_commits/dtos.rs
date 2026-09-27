use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::features::tables::ColumnInfo;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeltaCommitInfo {
    pub version: i64,
    pub timestamp: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_modification_timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct DeltaColumnInfos {
    pub columns: Option<Vec<ColumnInfo>>,
}

#[derive(Debug, Deserialize)]
pub struct DeltaCommitMetadataProperties {
    pub properties: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct DeltaMetadata {
    pub description: Option<String>,
    pub schema: Option<DeltaColumnInfos>,
    pub properties: Option<DeltaCommitMetadataProperties>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeltaCommitRequest {
    #[validate(length(min = 1))]
    pub table_id: String,
    #[validate(length(min = 1))]
    pub table_uri: String,
    pub commit_info: Option<DeltaCommitInfo>,
    pub latest_backfilled_version: Option<i64>,
    pub metadata: Option<DeltaMetadata>,
    pub uniform: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeltaGetCommitsRequest {
    #[validate(length(min = 1))]
    pub table_id: String,
    #[validate(length(min = 1))]
    pub table_uri: String,
    #[validate(range(min = 0))]
    pub start_version: i64,
    pub end_version: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DeltaGetCommitsResponse {
    pub commits: Vec<DeltaCommitInfo>,
    pub latest_table_version: i64,
}
