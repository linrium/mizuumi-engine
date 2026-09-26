use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::ExternalLocation;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateExternalLocationRequest {
    #[validate(length(min = 1, max = 255), regex(path = *EXTERNAL_LOCATION_NAME_REGEX))]
    pub name: String,
    #[validate(length(min = 1))]
    pub url: String,
    #[validate(length(min = 1))]
    pub credential_name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListExternalLocationsRequest {
    #[validate(range(min = 0, max = 1000))]
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateExternalLocationRequest {
    #[validate(length(min = 1))]
    pub url: Option<String>,
    #[validate(length(min = 1))]
    pub credential_name: Option<String>,
    pub comment: Option<String>,
    pub owner: Option<String>,
    #[validate(length(min = 1, max = 255), regex(path = *EXTERNAL_LOCATION_NAME_REGEX))]
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteExternalLocationRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ExternalLocationInfo {
    pub name: String,
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub credential_id: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListExternalLocationsResponse {
    pub external_locations: Vec<ExternalLocationInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl From<ExternalLocation> for ExternalLocationInfo {
    fn from(location: ExternalLocation) -> Self {
        Self {
            name: location.name,
            id: location.id,
            url: location.url,
            credential_name: non_empty(location.credential_name),
            comment: non_empty(location.comment),
            owner: non_empty(location.owner),
            credential_id: location.credential_id,
            created_at: location.created_at,
            created_by: non_empty(location.created_by),
            updated_at: location.updated_at,
            updated_by: non_empty(location.updated_by),
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

static EXTERNAL_LOCATION_NAME_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z0-9_@-]+$").expect("valid external location name regex")
});
