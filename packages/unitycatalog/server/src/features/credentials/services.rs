use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_sts::config::Builder as StsConfigBuilder;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::credentials as queries;

use crate::{config::VendingSettings, error::AppError};

use super::{
    dtos::{
        AwsIamRoleRequest, CreateCredentialRequest, CredentialInfo, DeleteCredentialRequest,
        ListCredentialsRequest, ListCredentialsResponse, RustfsServiceAccountRequest,
        UpdateCredentialRequest,
    },
    models::{AwsIamRole, Credential, CredentialKind, RustfsServiceAccount},
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const STORAGE_PURPOSE: &str = "STORAGE";
const AWS_IAM_ROLE_TYPE: &str = "AWS_IAM_ROLE";
const RUSTFS_SERVICE_ACCOUNT_TYPE: &str = "RUSTFS_SERVICE_ACCOUNT";
const DEFAULT_RUSTFS_ROLE_ARN: &str = "arn:aws:iam::000000000000:role/unitycatalog";

#[async_trait]
pub trait CredentialService: Send + Sync {
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
    ) -> Result<CredentialInfo, AppError>;
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
    ) -> Result<ListCredentialsResponse, AppError>;
    async fn get_credential(&self, name: String) -> Result<CredentialInfo, AppError>;
    async fn update_credential(
        &self,
        name: String,
        request: UpdateCredentialRequest,
    ) -> Result<CredentialInfo, AppError>;
    async fn delete_credential(
        &self,
        name: String,
        request: DeleteCredentialRequest,
    ) -> Result<(), AppError>;
}

pub struct DefaultCredentialService {
    pool: Pool,
    vending_settings: VendingSettings,
}

impl DefaultCredentialService {
    pub fn new(pool: Pool, vending_settings: VendingSettings) -> Self {
        Self {
            pool,
            vending_settings,
        }
    }
}

#[async_trait]
impl CredentialService for DefaultCredentialService {
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
    ) -> Result<CredentialInfo, AppError> {
        validate_purpose(request.purpose.as_deref())?;
        let (credential_type, credential) = self
            .credential_payload(request.aws_iam_role, request.rustfs_service_account)
            .await?;
        let purpose = request
            .purpose
            .unwrap_or_else(|| STORAGE_PURPOSE.to_string());
        let comment = request.comment.unwrap_or_default();
        let client = self.pool.get().await?;

        let row = queries::create_credential()
            .bind(
                &client,
                &request.name,
                &credential_type,
                &credential,
                &purpose,
                &comment,
            )
            .one()
            .await
            .map_err(map_credential_write_error)?;

        Ok(row_to_credential(row)?.into())
    }

    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
    ) -> Result<ListCredentialsResponse, AppError> {
        validate_purpose(request.purpose.as_deref())?;
        let page_size = page_size(request.max_results)?;
        let limit = i64::from(page_size);
        let page_token = request.page_token.unwrap_or_default();
        let purpose = request.purpose.unwrap_or_default();
        let client = self.pool.get().await?;

        let credentials = queries::list_credentials()
            .bind(&client, &page_token, &purpose, &limit)
            .all()
            .await?
            .into_iter()
            .map(row_to_credential)
            .collect::<Result<Vec<_>, _>>()?;

        let next_page_token = if credentials.len() == page_size as usize {
            credentials.last().map(|credential| credential.name.clone())
        } else {
            None
        };

        Ok(ListCredentialsResponse {
            credentials: credentials.into_iter().map(Into::into).collect(),
            next_page_token,
        })
    }

    async fn get_credential(&self, name: String) -> Result<CredentialInfo, AppError> {
        let client = self.pool.get().await?;
        let row = queries::get_credential()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("credential {name}")))?;

        Ok(row_to_credential(row)?.into())
    }

    async fn update_credential(
        &self,
        name: String,
        request: UpdateCredentialRequest,
    ) -> Result<CredentialInfo, AppError> {
        let client = self.pool.get().await?;
        let current = queries::get_credential()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("credential {name}")))?;
        let current = row_to_credential(current)?;

        let new_name = request.new_name.unwrap_or(current.name);
        let (credential_type, credential) =
            match (request.aws_iam_role, request.rustfs_service_account) {
                (None, None) => (String::new(), Value::Null),
                (aws_iam_role, rustfs_service_account) => {
                    self.credential_payload(aws_iam_role, rustfs_service_account)
                        .await?
                }
            };
        let comment = request.comment.unwrap_or(current.comment);
        let _owner = request.owner;

        let row = queries::update_credential()
            .bind(
                &client,
                &new_name,
                &credential_type,
                &credential,
                &comment,
                &name,
            )
            .opt()
            .await
            .map_err(map_credential_write_error)?
            .ok_or_else(|| AppError::NotFound(format!("credential {name}")))?;

        Ok(row_to_credential(row)?.into())
    }

    async fn delete_credential(
        &self,
        name: String,
        request: DeleteCredentialRequest,
    ) -> Result<(), AppError> {
        let _force = request.force.unwrap_or(false);
        let client = self.pool.get().await?;
        queries::delete_credential()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("credential {name}")))?;

        Ok(())
    }
}

impl DefaultCredentialService {
    async fn credential_payload(
        &self,
        aws_iam_role: Option<AwsIamRoleRequest>,
        rustfs_service_account: Option<RustfsServiceAccountRequest>,
    ) -> Result<(String, Value), AppError> {
        match (aws_iam_role, rustfs_service_account) {
            (Some(_), Some(_)) => Err(AppError::InvalidParameter(
                "only one of aws_iam_role or rustfs_service_account may be set".to_string(),
            )),
            (Some(role), None) => {
                if role.role_arn.is_empty() {
                    return Err(AppError::InvalidParameter(
                        "aws_iam_role.role_arn must not be empty".to_string(),
                    ));
                }

                Ok((
                    AWS_IAM_ROLE_TYPE.to_string(),
                    json!({
                        "role_arn": role.role_arn,
                        "external_id": uuid::Uuid::new_v4().to_string(),
                    }),
                ))
            }
            (None, Some(account)) => {
                let account = self.normalize_rustfs_service_account(account)?;
                self.validate_rustfs_service_account(&account).await?;
                Ok((
                    RUSTFS_SERVICE_ACCOUNT_TYPE.to_string(),
                    serde_json::to_value(account)?,
                ))
            }
            (None, None) => Err(AppError::InvalidParameter(
                "rustfs_service_account is required".to_string(),
            )),
        }
    }

    fn normalize_rustfs_service_account(
        &self,
        request: RustfsServiceAccountRequest,
    ) -> Result<RustfsServiceAccountPayload, AppError> {
        if request.access_key.is_empty() {
            return Err(AppError::InvalidParameter(
                "rustfs_service_account.access_key must not be empty".to_string(),
            ));
        }
        if request.secret_key.is_empty() {
            return Err(AppError::InvalidParameter(
                "rustfs_service_account.secret_key must not be empty".to_string(),
            ));
        }

        Ok(RustfsServiceAccountPayload {
            endpoint_url: request
                .endpoint_url
                .unwrap_or_else(|| self.vending_settings.endpoint_url.clone()),
            region: request
                .region
                .unwrap_or_else(|| self.vending_settings.region.clone()),
            access_key: request.access_key,
            secret_key: request.secret_key,
            role_arn: request
                .role_arn
                .unwrap_or_else(|| DEFAULT_RUSTFS_ROLE_ARN.to_string()),
            force_path_style: request
                .force_path_style
                .unwrap_or(self.vending_settings.force_path_style),
            duration_seconds: request
                .duration_seconds
                .unwrap_or(self.vending_settings.duration_seconds),
        })
    }

    async fn validate_rustfs_service_account(
        &self,
        account: &RustfsServiceAccountPayload,
    ) -> Result<(), AppError> {
        let credentials_provider = SharedCredentialsProvider::new(Credentials::new(
            account.access_key.clone(),
            account.secret_key.clone(),
            None,
            None,
            "rustfs-credential-config",
        ));
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(account.region.clone()))
            .endpoint_url(&account.endpoint_url)
            .credentials_provider(credentials_provider)
            .load()
            .await;
        let sts_config = StsConfigBuilder::from(&shared_config).build();
        aws_sdk_sts::Client::from_conf(sts_config)
            .assume_role()
            .role_arn(&account.role_arn)
            .role_session_name("unitycatalog-credential-validation")
            .duration_seconds(account.duration_seconds as i32)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, "failed to validate RustFS service account with STS");
                AppError::Sts
            })?;

        Ok(())
    }
}

fn validate_purpose(purpose: Option<&str>) -> Result<(), AppError> {
    if let Some(purpose) = purpose
        && purpose != STORAGE_PURPOSE
    {
        return Err(AppError::InvalidParameter(format!(
            "unsupported credential purpose: {purpose}"
        )));
    }

    Ok(())
}

fn page_size(max_results: Option<i32>) -> Result<i32, AppError> {
    match max_results {
        Some(value) if value < 0 => Err(AppError::InvalidParameter(
            "max_results must be greater than or equal to 0".to_string(),
        )),
        Some(0) | None => Ok(DEFAULT_PAGE_SIZE),
        Some(value) => Ok(value.min(DEFAULT_PAGE_SIZE)),
    }
}

fn map_credential_write_error(error: tokio_postgres::Error) -> AppError {
    if error.code() == Some(&SqlState::UNIQUE_VIOLATION) {
        AppError::Conflict("credential name".to_string())
    } else {
        AppError::Postgres(error)
    }
}

fn row_to_credential(row: impl IntoCredentialParts) -> Result<Credential, AppError> {
    let row = row.into_credential_parts();
    let kind = match row.credential_type.as_str() {
        AWS_IAM_ROLE_TYPE => {
            let role: AwsIamRolePayload = serde_json::from_value(row.credential)?;
            CredentialKind::AwsIamRole(AwsIamRole {
                role_arn: role.role_arn,
                external_id: role.external_id,
            })
        }
        RUSTFS_SERVICE_ACCOUNT_TYPE => {
            let account: RustfsServiceAccountPayload = serde_json::from_value(row.credential)?;
            CredentialKind::RustfsServiceAccount(RustfsServiceAccount {
                endpoint_url: account.endpoint_url,
                region: account.region,
                access_key: account.access_key,
                role_arn: account.role_arn,
                force_path_style: account.force_path_style,
                duration_seconds: account.duration_seconds,
            })
        }
        credential_type => {
            return Err(AppError::InvalidParameter(format!(
                "unsupported credential type: {credential_type}"
            )));
        }
    };

    Ok(Credential {
        name: row.name,
        kind,
        comment: row.comment,
        owner: row.owner,
        full_name: row.full_name,
        id: row.id,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: (row.updated_at > 0).then_some(row.updated_at),
        updated_by: row.updated_by,
        purpose: row.purpose,
    })
}

#[derive(Deserialize)]
struct AwsIamRolePayload {
    role_arn: String,
    external_id: String,
}

#[derive(Deserialize, Serialize)]
struct RustfsServiceAccountPayload {
    endpoint_url: String,
    region: String,
    access_key: String,
    secret_key: String,
    role_arn: String,
    force_path_style: bool,
    duration_seconds: u32,
}

struct CredentialParts {
    name: String,
    credential_type: String,
    credential: Value,
    comment: String,
    owner: String,
    full_name: String,
    id: String,
    created_at: i64,
    created_by: String,
    updated_at: i64,
    updated_by: String,
    purpose: String,
}

trait IntoCredentialParts {
    fn into_credential_parts(self) -> CredentialParts;
}

macro_rules! impl_into_credential_parts {
    ($type:ty) => {
        impl IntoCredentialParts for $type {
            fn into_credential_parts(self) -> CredentialParts {
                CredentialParts {
                    name: self.name,
                    credential_type: self.credential_type,
                    credential: self.credential,
                    comment: self.comment,
                    owner: self.owner,
                    full_name: self.full_name,
                    id: self.id,
                    created_at: self.created_at,
                    created_by: self.created_by,
                    updated_at: self.updated_at,
                    updated_by: self.updated_by,
                    purpose: self.purpose,
                }
            }
        }
    };
}

impl_into_credential_parts!(queries::CreateCredential);
impl_into_credential_parts!(queries::ListCredentials);
impl_into_credential_parts!(queries::GetCredential);
impl_into_credential_parts!(queries::UpdateCredential);
