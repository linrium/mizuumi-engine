use std::{collections::BTreeMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use super::models::Table;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ColumnInfo {
    pub name: Option<String>,
    pub type_text: Option<String>,
    pub type_json: Option<String>,
    pub type_name: Option<String>,
    pub type_precision: Option<i32>,
    pub type_scale: Option<i32>,
    pub type_interval_type: Option<String>,
    pub position: Option<i32>,
    pub comment: Option<String>,
    pub nullable: Option<bool>,
    pub partition_index: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTableRequest {
    #[validate(length(min = 1, max = 255), regex(path = *TABLE_NAME_REGEX))]
    pub name: String,
    #[validate(length(min = 1))]
    pub catalog_name: String,
    #[validate(length(min = 1))]
    pub schema_name: String,
    pub table_type: String,
    pub data_source_format: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub storage_location: Option<String>,
    pub comment: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
    pub view_definition: Option<String>,
    pub view_dependencies: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateStagingTableRequest {
    #[validate(length(min = 1, max = 255), regex(path = *TABLE_NAME_REGEX))]
    pub name: String,
    #[validate(length(min = 1))]
    pub catalog_name: String,
    #[validate(length(min = 1))]
    pub schema_name: String,
}

#[derive(Debug, Serialize)]
pub struct StagingTableInfo {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub id: String,
    pub staging_location: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListTablesRequest {
    #[validate(length(min = 1))]
    pub catalog_name: String,
    #[validate(length(min = 1))]
    pub schema_name: String,
    #[validate(range(min = 0))]
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
    pub omit_properties: Option<bool>,
    pub omit_columns: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetTableRequest {
    pub read_streaming_table_as_managed: Option<bool>,
    pub read_materialized_view_as_managed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<ColumnInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    pub table_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_definition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_dependencies: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ListTablesResponse {
    pub tables: Vec<TableInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl Table {
    pub fn into_info(self, omit_properties: bool, omit_columns: bool) -> TableInfo {
        TableInfo {
            name: self.name,
            catalog_name: self.catalog_name,
            schema_name: self.schema_name,
            table_type: self.table_type,
            data_source_format: non_empty(self.data_source_format),
            columns: (!omit_columns).then_some(self.columns),
            storage_location: non_empty(self.storage_location),
            comment: non_empty(self.comment),
            properties: if omit_properties {
                BTreeMap::new()
            } else {
                self.properties
            },
            owner: non_empty(self.owner),
            created_at: self.created_at,
            created_by: non_empty(self.created_by),
            updated_at: self.updated_at,
            updated_by: non_empty(self.updated_by),
            table_id: self.table_id,
            view_definition: non_empty(self.view_definition),
            view_dependencies: self.view_dependencies,
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

static TABLE_NAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_@-]+$").expect("valid table name regex"));
