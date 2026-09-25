use std::{collections::BTreeMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::Catalog;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCatalogRequest {
    #[validate(length(min = 1, max = 255), regex(path = *CATALOG_NAME_REGEX))]
    pub name: String,
    pub comment: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
    pub storage_root: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListCatalogsRequest {
    pub page_token: Option<String>,
    #[validate(range(min = 0, max = 1000))]
    pub max_results: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCatalogRequest {
    pub comment: Option<String>,
    pub properties: Option<BTreeMap<String, String>>,
    #[validate(length(min = 1, max = 255), regex(path = *CATALOG_NAME_REGEX))]
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteCatalogRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CatalogInfo {
    pub name: String,
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
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_location: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListCatalogsResponse {
    pub catalogs: Vec<CatalogInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl From<Catalog> for CatalogInfo {
    fn from(catalog: Catalog) -> Self {
        Self {
            name: catalog.name,
            comment: non_empty(catalog.comment),
            properties: catalog.properties,
            owner: non_empty(catalog.owner),
            created_at: catalog.created_at,
            created_by: non_empty(catalog.created_by),
            updated_at: catalog.updated_at,
            updated_by: non_empty(catalog.updated_by),
            id: catalog.id,
            storage_root: non_empty(catalog.storage_root),
            storage_location: non_empty(catalog.storage_location),
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

static CATALOG_NAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_@-]+$").expect("valid catalog name regex"));
