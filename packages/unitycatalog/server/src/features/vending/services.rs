use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_sts::config::Builder as StsConfigBuilder;
use deadpool_postgres::Pool;
use serde_json::json;
use unitycatalog_queries::queries::external_locations as location_queries;
use url::Url;

use crate::{
    config::VendingSettings, error::AppError,
    features::external_locations::normalize_external_location_url,
};

use super::{
    dtos::{
        GenerateTemporaryPathCredentialRequest, ListBucketsRequest, ListBucketsResponse,
        PathOperation, TemporaryCredentialsResponse,
    },
    models::{Bucket, VendedCredentials},
};

#[async_trait]
pub trait VendingService: Send + Sync {
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialRequest,
    ) -> Result<TemporaryCredentialsResponse, AppError>;
    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, AppError>;
}

pub struct DefaultVendingService {
    pool: Pool,
    settings: VendingSettings,
}

impl DefaultVendingService {
    pub fn new(pool: Pool, settings: VendingSettings) -> anyhow::Result<Self> {
        Ok(Self { pool, settings })
    }

    async fn assume_role(
        &self,
        duration_seconds: u32,
        policy: Option<&str>,
    ) -> Result<VendedCredentials, AppError> {
        let credentials_provider = SharedCredentialsProvider::new(Credentials::new(
            self.settings.access_key.clone(),
            self.settings.secret_key.clone(),
            None,
            None,
            "rustfs-sts-config",
        ));
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(self.settings.region.clone()))
            .endpoint_url(&self.settings.endpoint_url)
            .credentials_provider(credentials_provider)
            .load()
            .await;
        let sts_config = StsConfigBuilder::from(&shared_config).build();
        let mut request = aws_sdk_sts::Client::from_conf(sts_config)
            .assume_role()
            .role_arn("arn:aws:iam::000000000000:role/unitycatalog")
            .role_session_name("unitycatalog-vending")
            .duration_seconds(duration_seconds as i32);
        if let Some(policy) = policy {
            request = request.policy(policy);
        }
        let response = request.send().await.map_err(|error| {
            tracing::error!(%error, "failed to assume role with RustFS STS");
            AppError::Sts
        })?;
        let credentials = response.credentials().ok_or(AppError::StsCredentials)?;
        let expiration_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .saturating_add(u128::from(duration_seconds) * 1000)
            .min(i64::MAX as u128) as i64;
        Ok(VendedCredentials {
            credentials: Credentials::new(
                credentials.access_key_id(),
                credentials.secret_access_key(),
                Some(credentials.session_token().to_owned()),
                None,
                "rustfs-sts",
            ),
            expiration_time,
        })
    }

    async fn list_buckets_with_credentials(
        &self,
        credentials: Credentials,
    ) -> Result<Vec<Bucket>, AppError> {
        let credentials_provider = SharedCredentialsProvider::new(credentials);
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(self.settings.region.clone()))
            .endpoint_url(&self.settings.endpoint_url)
            .credentials_provider(credentials_provider)
            .load()
            .await;
        let s3_config = S3ConfigBuilder::from(&shared_config)
            .force_path_style(self.settings.force_path_style)
            .build();
        let response = aws_sdk_s3::Client::from_conf(s3_config)
            .list_buckets()
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to list buckets with vended credentials");
                AppError::S3
            })?;

        Ok(response
            .buckets()
            .iter()
            .filter_map(|bucket| {
                bucket.name().map(|name| Bucket {
                    name: name.to_owned(),
                })
            })
            .collect())
    }
}

#[async_trait]
impl VendingService for DefaultVendingService {
    async fn generate_temporary_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialRequest,
    ) -> Result<TemporaryCredentialsResponse, AppError> {
        if matches!(request.operation, PathOperation::UnknownPathOperation) {
            return Err(AppError::InvalidParameter(
                "operation must not be UNKNOWN_PATH_OPERATION".to_string(),
            ));
        }

        let url = normalize_external_location_url(&request.url)?;
        let client = self.pool.get().await?;
        let _external_location = location_queries::find_external_location_for_path()
            .bind(&client, &url)
            .opt()
            .await?
            .ok_or_else(|| {
                AppError::FailedPrecondition(format!(
                    "S3 bucket configuration not found for path: {url}"
                ))
            })?;
        let policy = session_policy(&url, request.operation)?;
        let vended = self
            .assume_role(self.settings.duration_seconds, Some(&policy))
            .await?;

        Ok(TemporaryCredentialsResponse::new(vended, url))
    }

    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, AppError> {
        let duration_seconds = request
            .duration_seconds
            .unwrap_or(self.settings.duration_seconds);
        let credentials = self.assume_role(duration_seconds, None).await?;
        let buckets = self
            .list_buckets_with_credentials(credentials.credentials)
            .await?;

        Ok(ListBucketsResponse::from_buckets(buckets))
    }
}

fn session_policy(url: &str, operation: PathOperation) -> Result<String, AppError> {
    let url = Url::parse(url)
        .map_err(|_| AppError::InvalidParameter(format!("unsupported path: {url}")))?;
    if url.scheme() != "s3" {
        return Err(AppError::InvalidParameter(format!(
            "temporary credentials are not supported for URI scheme: {}",
            url.scheme()
        )));
    }
    let bucket = url
        .host_str()
        .ok_or_else(|| AppError::InvalidParameter(format!("S3 path has no bucket: {url}")))?;
    let prefix = url.path().trim_matches('/');
    let object_resource = if prefix.is_empty() {
        format!("arn:aws:s3:::{bucket}/*")
    } else {
        format!("arn:aws:s3:::{bucket}/{prefix}/*")
    };
    let mut bucket_actions = vec!["s3:GetBucketLocation", "s3:ListBucket"];
    let mut object_actions = vec!["s3:GetObject"];
    if matches!(
        operation,
        PathOperation::PathReadWrite | PathOperation::PathCreateTable
    ) {
        bucket_actions.push("s3:ListBucketMultipartUploads");
        object_actions.extend([
            "s3:PutObject",
            "s3:DeleteObject",
            "s3:AbortMultipartUpload",
            "s3:ListMultipartUploadParts",
        ]);
    }

    Ok(json!({
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Action": bucket_actions,
                "Resource": [format!("arn:aws:s3:::{bucket}")],
            },
            {
                "Effect": "Allow",
                "Action": object_actions,
                "Resource": [object_resource],
            }
        ]
    })
    .to_string())
}
