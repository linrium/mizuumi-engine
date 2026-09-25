use std::collections::BTreeMap;

use serde_json::Value;

use super::dtos::ColumnInfo;

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_type: String,
    pub data_source_format: String,
    pub columns: Vec<ColumnInfo>,
    pub storage_location: String,
    pub comment: String,
    pub properties: BTreeMap<String, String>,
    pub owner: String,
    pub created_at: i64,
    pub created_by: String,
    pub updated_at: Option<i64>,
    pub updated_by: String,
    pub table_id: String,
    pub view_definition: String,
    pub view_dependencies: Option<Value>,
}
