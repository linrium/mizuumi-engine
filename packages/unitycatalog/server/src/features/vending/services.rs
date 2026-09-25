use std::time::SystemTime;

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4,
};
use aws_smithy_runtime_api::client::identity::Identity;
use http::{HeaderMap, Method, Request};
use reqwest::Client as HttpClient;
use url::form_urlencoded;

use crate::{config::VendingSettings, error::AppError};

use super::{
    dtos::{ListBucketsRequest, ListBucketsResponse},
    models::{AssumeRoleResponse, Bucket, TemporaryCredentials},
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
    http_client: HttpClient,
}

impl DefaultVendingService {
    pub fn new(settings: VendingSettings) -> Self {
        Self {
            settings,
            http_client: HttpClient::new(),
        }
    }

    async fn assume_role(&self, duration_seconds: u32) -> Result<TemporaryCredentials, AppError> {
        let body = form_urlencoded::Serializer::new(String::new())
            .append_pair("Action", "AssumeRole")
            .append_pair("Version", "2011-06-15")
            .append_pair("DurationSeconds", &duration_seconds.to_string())
            .finish();
        let headers = self.sign_sts_request(&body)?;

        let response = self
            .http_client
            .post(&self.settings.endpoint_url)
            .headers(headers)
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response: AssumeRoleResponse =
            quick_xml::de::from_str(&response).map_err(|_| AppError::StsResponse)?;

        Ok(response.result.credentials)
    }

    fn sign_sts_request(&self, body: &str) -> Result<HeaderMap, AppError> {
        let identity: Identity = Credentials::new(
            self.settings.access_key.clone(),
            self.settings.secret_key.clone(),
            None,
            None,
            "rustfs-sts-config",
        )
        .into();
        let signing_params = v4::SigningParams::builder()
            .identity(&identity)
            .region(&self.settings.region)
            .name("s3")
            .time(SystemTime::now())
            .settings(SigningSettings::default())
            .build()
            .map_err(|_| AppError::StsSigning)?
            .into();
        let host_header = self.host_header()?;
        let headers = [
            ("content-type", "application/x-www-form-urlencoded"),
            ("host", host_header.as_str()),
        ];
        let signable_request = SignableRequest::new(
            "POST",
            &self.settings.endpoint_url,
            headers.into_iter(),
            SignableBody::Bytes(body.as_bytes()),
        )
        .map_err(|_| AppError::StsSigning)?;

        let mut request = Request::builder()
            .method(Method::POST)
            .uri(&self.settings.endpoint_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(())
            .map_err(|_| AppError::StsSigning)?;
        let (instructions, _) = sign(signable_request, &signing_params)
            .map_err(|_| AppError::StsSigning)?
            .into_parts();
        instructions.apply_to_request_http1x(&mut request);

        Ok(request.headers().clone())
    }

    fn host_header(&self) -> Result<String, AppError> {
        let url = url::Url::parse(&self.settings.endpoint_url).map_err(|_| AppError::StsSigning)?;
        url.host_str()
            .map(|host| match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host.to_owned(),
            })
            .ok_or(AppError::StsSigning)
    }

    async fn list_buckets_with_credentials(
        &self,
        credentials: TemporaryCredentials,
    ) -> Result<Vec<Bucket>, AppError> {
        let credentials_provider = SharedCredentialsProvider::new(Credentials::new(
            credentials.access_key_id,
            credentials.secret_access_key,
            Some(credentials.session_token),
            None,
            "rustfs-sts",
        ));
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
