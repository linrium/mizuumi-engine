use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::VendedCredentials;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PathOperation {
    UnknownPathOperation,
    PathRead,
    PathReadWrite,
    PathCreateTable,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableOperation {
    UnknownTableOperation,
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VolumeOperation {
    UnknownVolumeOperation,
    ReadVolume,
    WriteVolume,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelVersionOperation {
    UnknownModelVersionOperation,
    ReadModelVersion,
    ReadWriteModelVersion,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemporaryPathCredentialRequest {
    #[validate(length(min = 1))]
    pub url: String,
    pub operation: PathOperation,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemporaryTableCredentialRequest {
    #[validate(length(min = 1))]
    pub table_id: String,
    pub operation: TableOperation,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemporaryVolumeCredentialRequest {
    #[validate(length(min = 1))]
    pub volume_id: String,
    pub operation: VolumeOperation,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemporaryModelVersionCredentialRequest {
    #[validate(length(min = 1))]
    pub catalog_name: String,
    #[validate(length(min = 1))]
    pub schema_name: String,
    #[validate(length(min = 1))]
    pub model_name: String,
    pub version: i64,
    pub operation: ModelVersionOperation,
}

#[derive(Serialize)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

#[derive(Serialize)]
pub struct TemporaryCredentials {
    pub aws_temp_credentials: AwsCredentials,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_user_delegation_sas: Option<AzureUserDelegationSas>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_token: Option<GcpOauthToken>,
    pub expiration_time: i64,
    pub url: String,
}

#[derive(Serialize)]
pub struct AzureUserDelegationSas {
    pub sas_token: String,
}

#[derive(Serialize)]
pub struct GcpOauthToken {
    pub oauth_token: String,
}

impl TemporaryCredentials {
    pub fn from_aws(vended: VendedCredentials, url: String) -> Self {
        Self {
            aws_temp_credentials: AwsCredentials {
                access_key_id: vended.credentials.access_key_id().to_owned(),
                secret_access_key: vended.credentials.secret_access_key().to_owned(),
                session_token: vended
                    .credentials
                    .session_token()
                    .unwrap_or_default()
                    .to_owned(),
            },
            azure_user_delegation_sas: None,
            gcp_oauth_token: None,
            expiration_time: vended.expiration_time,
            url,
        }
    }
}
