use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_sts::config::Builder as StsConfigBuilder;
use deadpool_postgres::Pool;
use serde_json::json;
use unitycatalog_queries::queries::{
    external_locations as location_queries, temporary_credentials as credential_queries,
};
use url::Url;

use crate::{
    config::VendingSettings, error::AppError,
    features::external_locations::normalize_external_location_url,
};

use super::{
    dtos::{
        GenerateTemporaryModelVersionCredentialRequest, GenerateTemporaryPathCredentialRequest,
        GenerateTemporaryTableCredentialRequest, GenerateTemporaryVolumeCredentialRequest,
        ModelVersionOperation, PathOperation, TableOperation, TemporaryCredentials,
        VolumeOperation,
    },
    models::VendedCredentials,
};

#[derive(Clone, Copy)]
enum AccessMode {
    Read,
    ReadWrite,
}

#[async_trait]
pub trait TemporaryCredentialsService: Send + Sync {
    async fn generate_model_version_credentials(
        &self,
        request: GenerateTemporaryModelVersionCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError>;
    async fn generate_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError>;
    async fn generate_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError>;
    async fn generate_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError>;
}

pub struct DefaultTemporaryCredentialsService {
    pool: Pool,
    settings: VendingSettings,
}

impl DefaultTemporaryCredentialsService {
    pub fn new(pool: Pool, settings: VendingSettings) -> Self {
        Self { pool, settings }
    }

    async fn vend(
        &self,
        url: String,
        access_mode: AccessMode,
    ) -> Result<TemporaryCredentials, AppError> {
        let url = normalize_external_location_url(&url)?;
        let client = self.pool.get().await?;
        location_queries::find_external_location_for_path()
            .bind(&client, &url)
            .opt()
            .await?
            .ok_or_else(|| {
                AppError::FailedPrecondition(format!(
                    "storage credential configuration not found for path: {url}"
                ))
            })?;
        drop(client);

        let policy = session_policy(&url, access_mode)?;
        let vended = self.assume_role(&policy).await?;
        Ok(TemporaryCredentials::from_aws(vended, url))
    }

    async fn assume_role(&self, policy: &str) -> Result<VendedCredentials, AppError> {
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
            .role_session_name("unitycatalog-temporary-credentials")
            .duration_seconds(self.settings.duration_seconds as i32)
            .policy(policy)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to assume role with RustFS STS");
                AppError::Sts
            })?;
        let credentials = response.credentials().ok_or(AppError::StsCredentials)?;
        let expiration_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .saturating_add(u128::from(self.settings.duration_seconds) * 1000)
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
}

#[async_trait]
impl TemporaryCredentialsService for DefaultTemporaryCredentialsService {
    async fn generate_model_version_credentials(
        &self,
        request: GenerateTemporaryModelVersionCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError> {
        let access_mode = model_version_access_mode(request.operation)?;
        let client = self.pool.get().await?;
        let model_version = credential_queries::get_model_version_storage_location()
            .bind(
                &client,
                &request.catalog_name,
                &request.schema_name,
                &request.model_name,
                &request.version,
            )
            .opt()
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "model version {}.{}.{}/{}",
                    request.catalog_name, request.schema_name, request.model_name, request.version
                ))
            })?;
        validate_model_version_status(&model_version.status, request.operation)?;
        drop(client);

        self.vend(model_version.storage_location, access_mode).await
    }

    async fn generate_path_credentials(
        &self,
        request: GenerateTemporaryPathCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError> {
        self.vend(request.url, path_access_mode(request.operation)?)
            .await
    }

    async fn generate_table_credentials(
        &self,
        request: GenerateTemporaryTableCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError> {
        let access_mode = table_access_mode(request.operation)?;
        let client = self.pool.get().await?;
        let storage_location = credential_queries::get_table_storage_location()
            .bind(&client, &request.table_id)
            .opt()
            .await?;
        let storage_location = match storage_location {
            Some(location) => location,
            None => client
                .query_opt(
                    "SELECT staging_location FROM uc_staging_tables \
                     WHERE id = $1 AND finalized_at IS NULL",
                    &[&request.table_id],
                )
                .await?
                .map(|row| row.get(0))
                .ok_or_else(|| AppError::NotFound(format!("table {}", request.table_id)))?,
        };
        drop(client);

        self.vend(storage_location, access_mode).await
    }

    async fn generate_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialRequest,
    ) -> Result<TemporaryCredentials, AppError> {
        let access_mode = volume_access_mode(request.operation)?;
        let client = self.pool.get().await?;
        let storage_location = credential_queries::get_volume_storage_location()
            .bind(&client, &request.volume_id)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("volume {}", request.volume_id)))?;
        drop(client);

        self.vend(storage_location, access_mode).await
    }
}

fn path_access_mode(operation: PathOperation) -> Result<AccessMode, AppError> {
    match operation {
        PathOperation::PathRead => Ok(AccessMode::Read),
        PathOperation::PathReadWrite | PathOperation::PathCreateTable => Ok(AccessMode::ReadWrite),
        PathOperation::UnknownPathOperation => Err(unknown_operation("UNKNOWN_PATH_OPERATION")),
    }
}

fn table_access_mode(operation: TableOperation) -> Result<AccessMode, AppError> {
    match operation {
        TableOperation::Read => Ok(AccessMode::Read),
        TableOperation::ReadWrite => Ok(AccessMode::ReadWrite),
        TableOperation::UnknownTableOperation => Err(unknown_operation("UNKNOWN_TABLE_OPERATION")),
    }
}

fn volume_access_mode(operation: VolumeOperation) -> Result<AccessMode, AppError> {
    match operation {
        VolumeOperation::ReadVolume => Ok(AccessMode::Read),
        VolumeOperation::WriteVolume => Ok(AccessMode::ReadWrite),
        VolumeOperation::UnknownVolumeOperation => {
            Err(unknown_operation("UNKNOWN_VOLUME_OPERATION"))
        }
    }
}

fn model_version_access_mode(operation: ModelVersionOperation) -> Result<AccessMode, AppError> {
    match operation {
        ModelVersionOperation::ReadModelVersion => Ok(AccessMode::Read),
        ModelVersionOperation::ReadWriteModelVersion => Ok(AccessMode::ReadWrite),
        ModelVersionOperation::UnknownModelVersionOperation => {
            Err(unknown_operation("UNKNOWN_MODEL_VERSION_OPERATION"))
        }
    }
}

fn unknown_operation(operation: &str) -> AppError {
    AppError::InvalidParameter(format!("unknown operation in the request: {operation}"))
}

fn validate_model_version_status(
    status: &str,
    operation: ModelVersionOperation,
) -> Result<(), AppError> {
    match status {
        "FAILED_REGISTRATION" | "MODEL_VERSION_STATUS_UNKNOWN" => Err(AppError::InvalidParameter(
            format!("cannot request credentials on a model version with status {status}"),
        )),
        "PENDING_REGISTRATION" | "READY" => {
            if status == "READY"
                && matches!(operation, ModelVersionOperation::ReadWriteModelVersion)
            {
                Err(AppError::InvalidParameter(
                    "cannot request read/write credentials on a finalized model version"
                        .to_string(),
                ))
            } else {
                Ok(())
            }
        }
        _ => Err(AppError::InvalidParameter(format!(
            "unknown model version status: {status}"
        ))),
    }
}

fn session_policy(url: &str, access_mode: AccessMode) -> Result<String, AppError> {
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
    if prefix.is_empty() {
        return Err(AppError::InvalidParameter(format!(
            "storage location must include a non-empty path prefix: {url}"
        )));
    }

    let prefix = escape_iam_special_characters(prefix);
    let mut object_actions = vec!["s3:GetObject"];
    if matches!(access_mode, AccessMode::ReadWrite) {
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
                "Action": object_actions,
                "Resource": [
                    format!("arn:aws:s3:::{bucket}/{prefix}"),
                    format!("arn:aws:s3:::{bucket}/{prefix}/*")
                ],
            },
            {
                "Effect": "Allow",
                "Action": ["s3:ListBucket"],
                "Resource": [format!("arn:aws:s3:::{bucket}")],
                "Condition": {
                    "StringLike": {
                        "s3:prefix": [
                            prefix,
                            format!("{prefix}/"),
                            format!("{prefix}/*")
                        ]
                    }
                }
            }
        ]
    })
    .to_string())
}

fn escape_iam_special_characters(prefix: &str) -> String {
    prefix
        .replace('$', "${$}")
        .replace('*', "${*}")
        .replace('?', "${?}")
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn read_policy_is_scoped_and_read_only() {
        let policy: Value = serde_json::from_str(
            &session_policy("s3://bucket/some/prefix", AccessMode::Read).unwrap(),
        )
        .unwrap();

        assert_eq!(
            policy["Statement"][0]["Resource"][1],
            "arn:aws:s3:::bucket/some/prefix/*"
        );
        assert_eq!(
            policy["Statement"][1]["Condition"]["StringLike"]["s3:prefix"][0],
            "some/prefix"
        );
        assert_eq!(policy.to_string().contains("s3:PutObject"), false);
    }

    #[test]
    fn write_policy_includes_mutating_actions() {
        let policy = session_policy("s3://bucket/prefix", AccessMode::ReadWrite).unwrap();

        assert!(policy.contains("s3:PutObject"));
        assert!(policy.contains("s3:DeleteObject"));
    }

    #[test]
    fn storage_root_is_rejected() {
        assert!(matches!(
            session_policy("s3://bucket", AccessMode::Read),
            Err(AppError::InvalidParameter(_))
        ));
    }

    #[test]
    fn policy_escapes_iam_wildcards_in_path() {
        let policy = session_policy("s3://bucket/prefix*", AccessMode::Read).unwrap();

        assert!(policy.contains("prefix${*}"));
        assert!(!policy.contains("prefix*/*"));
    }

    #[test]
    fn finalized_model_version_rejects_write_credentials() {
        assert!(matches!(
            validate_model_version_status("READY", ModelVersionOperation::ReadWriteModelVersion),
            Err(AppError::InvalidParameter(_))
        ));
        assert!(
            validate_model_version_status("READY", ModelVersionOperation::ReadModelVersion).is_ok()
        );
    }
}
