use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_sts::config::Builder as StsConfigBuilder;

use crate::{config::VendingSettings, error::AppError};

use super::{
    dtos::{ListBucketsRequest, ListBucketsResponse},
    models::Bucket,
};

#[async_trait]
pub trait VendingService: Send + Sync {
    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, AppError>;
}

pub struct DefaultVendingService {
    settings: VendingSettings,
}

impl DefaultVendingService {
    pub fn new(settings: VendingSettings) -> anyhow::Result<Self> {
        Ok(Self { settings })
    }

    async fn assume_role(&self, duration_seconds: u32) -> Result<Credentials, AppError> {
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
        let response = aws_sdk_sts::Client::from_conf(sts_config)
            .assume_role()
            .role_arn("arn:aws:iam::000000000000:role/unitycatalog")
            .role_session_name("unitycatalog-vending")
            .duration_seconds(duration_seconds as i32)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to assume role with RustFS STS");
                AppError::Sts
            })?;
        let credentials = response.credentials().ok_or(AppError::StsCredentials)?;
        Ok(Credentials::new(
            credentials.access_key_id(),
            credentials.secret_access_key(),
            Some(credentials.session_token().to_owned()),
            None,
            "rustfs-sts",
        ))
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
    async fn list_buckets(
        &self,
        request: ListBucketsRequest,
    ) -> Result<ListBucketsResponse, AppError> {
        let duration_seconds = request
            .duration_seconds
            .unwrap_or(self.settings.duration_seconds);
        let credentials = self.assume_role(duration_seconds).await?;
        let buckets = self.list_buckets_with_credentials(credentials).await?;

        Ok(ListBucketsResponse::from_buckets(buckets))
    }
}
