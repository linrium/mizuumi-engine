use serde::{Deserialize, Serialize};
use validator::Validate;

use super::models::{Bucket, VendedCredentials};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PathOperation {
    UnknownPathOperation,
    PathRead,
    PathReadWrite,
    PathCreateTable,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemporaryPathCredentialRequest {
    #[validate(length(min = 1))]
    pub url: String,
    pub operation: PathOperation,
}

#[derive(Serialize)]
pub struct AwsTemporaryCredentialsResponse {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

#[derive(Serialize)]
pub struct TemporaryCredentialsResponse {
    pub aws_temp_credentials: AwsTemporaryCredentialsResponse,
    pub expiration_time: i64,
    pub url: String,
}

impl TemporaryCredentialsResponse {
    pub fn new(vended: VendedCredentials, url: String) -> Self {
        Self {
            aws_temp_credentials: AwsTemporaryCredentialsResponse {
                access_key_id: vended.credentials.access_key_id().to_owned(),
                secret_access_key: vended.credentials.secret_access_key().to_owned(),
                session_token: vended
                    .credentials
                    .session_token()
                    .unwrap_or_default()
                    .to_owned(),
            },
            expiration_time: vended.expiration_time,
            url,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListBucketsRequest {
    #[validate(range(min = 900, max = 43200))]
    pub duration_seconds: Option<u32>,
}

#[derive(Serialize)]
pub struct BucketResponse {
    pub name: String,
}

impl From<Bucket> for BucketResponse {
    fn from(bucket: Bucket) -> Self {
        Self { name: bucket.name }
    }
}

#[derive(Serialize)]
pub struct ListBucketsResponse {
    pub buckets: Vec<BucketResponse>,
}

impl ListBucketsResponse {
    pub fn from_buckets(buckets: Vec<Bucket>) -> Self {
        Self {
            buckets: buckets.into_iter().map(Into::into).collect(),
        }
    }
}
