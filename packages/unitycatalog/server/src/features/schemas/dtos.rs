use std::{collections::BTreeMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::Schema;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSchemaRequest {
    #[validate(length(min = 1, max = 255), regex(path = *SCHEMA_NAME_REGEX))]
    pub name: String,
    #[validate(length(min = 1))]
    pub catalog_name: String,
    pub comment: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
    pub storage_root: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListSchemasRequest {
    #[validate(length(min = 1))]
    pub catalog_name: String,
    #[validate(range(min = 0, max = 1000))]
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSchemaRequest {
    pub comment: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
    #[validate(length(min = 1, max = 255), regex(path = *SCHEMA_NAME_REGEX))]
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteSchemaRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SchemaInfo {
    pub name: String,
    pub catalog_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
    pub full_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    pub schema_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListSchemasResponse {
    pub schemas: Vec<SchemaInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl From<Schema> for SchemaInfo {
    fn from(schema: Schema) -> Self {
        Self {
            name: schema.name,
            catalog_name: schema.catalog_name,
            comment: non_empty(schema.comment),
            properties: schema.properties,
            full_name: schema.full_name,
            owner: non_empty(schema.owner),
            created_at: schema.created_at,
            created_by: non_empty(schema.created_by),
            updated_at: schema.updated_at,
            updated_by: non_empty(schema.updated_by),
            schema_id: schema.schema_id,
            storage_root: non_empty(schema.storage_root),
            storage_location: non_empty(schema.storage_location),
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

static SCHEMA_NAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_@-]+$").expect("valid schema name regex"));
