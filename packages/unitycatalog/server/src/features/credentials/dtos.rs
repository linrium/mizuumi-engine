use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::Credential;

#[derive(Debug, Deserialize, Serialize)]
pub struct AwsIamRoleRequest {
    pub role_arn: String,
}

#[derive(Debug, Serialize)]
pub struct AwsIamRoleResponse {
    pub role_arn: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unity_catalog_iam_arn: Option<String>,
    pub external_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCredentialRequest {
    #[validate(length(min = 1, max = 255), regex(path = *CREDENTIAL_NAME_REGEX))]
    pub name: String,
    pub comment: Option<String>,
    pub aws_iam_role: Option<AwsIamRoleRequest>,
    pub purpose: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListCredentialsRequest {
    #[validate(range(min = 0))]
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
    pub purpose: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCredentialRequest {
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub aws_iam_role: Option<AwsIamRoleRequest>,
    #[validate(length(min = 1, max = 255), regex(path = *CREDENTIAL_NAME_REGEX))]
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteCredentialRequest {
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CredentialInfo {
    pub name: String,
    pub aws_iam_role: AwsIamRoleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub full_name: String,
    pub id: String,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    pub purpose: String,
}

#[derive(Debug, Serialize)]
pub struct ListCredentialsResponse {
    pub credentials: Vec<CredentialInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl From<Credential> for CredentialInfo {
    fn from(credential: Credential) -> Self {
        Self {
            name: credential.name,
            aws_iam_role: AwsIamRoleResponse {
                role_arn: credential.aws_iam_role.role_arn,
                unity_catalog_iam_arn: None,
                external_id: credential.aws_iam_role.external_id,
            },
            comment: non_empty(credential.comment),
            owner: non_empty(credential.owner),
            full_name: credential.full_name,
            id: credential.id,
            created_at: credential.created_at,
            created_by: non_empty(credential.created_by),
            updated_at: credential.updated_at,
            updated_by: non_empty(credential.updated_by),
            purpose: credential.purpose,
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

static CREDENTIAL_NAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_@-]+$").expect("valid credential name regex"));
