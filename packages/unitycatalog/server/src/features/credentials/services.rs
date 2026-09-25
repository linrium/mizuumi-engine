use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::Value;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::credentials as queries;

use crate::error::AppError;

use super::{
    dtos::{
        CreateCredentialRequest, CredentialInfo, DeleteCredentialRequest, ListCredentialsRequest,
        ListCredentialsResponse, UpdateCredentialRequest,
    },
    models::{AwsIamRole, Credential},
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const STORAGE_PURPOSE: &str = "STORAGE";

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
}

impl DefaultCredentialService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CredentialService for DefaultCredentialService {
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
    ) -> Result<CredentialInfo, AppError> {
        let role_arn = request
            .aws_iam_role
            .ok_or_else(|| AppError::InvalidParameter("aws_iam_role is required".to_string()))?
            .role_arn;
        if role_arn.is_empty() {
            return Err(AppError::InvalidParameter(
                "aws_iam_role.role_arn must not be empty".to_string(),
            ));
        }
        validate_purpose(request.purpose.as_deref())?;
        let purpose = request
            .purpose
            .unwrap_or_else(|| STORAGE_PURPOSE.to_string());
        let comment = request.comment.unwrap_or_default();
        let client = self.pool.get().await?;

        let row = queries::create_credential()
            .bind(&client, &request.name, &role_arn, &purpose, &comment)
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
        let role_arn = match request.aws_iam_role {
            Some(role) if role.role_arn.is_empty() => {
                return Err(AppError::InvalidParameter(
                    "aws_iam_role.role_arn must not be empty".to_string(),
                ));
            }
            Some(role) => role.role_arn,
            None => String::new(),
        };
        let comment = request.comment.unwrap_or(current.comment);
        let _owner = request.owner;

        let row = queries::update_credential()
            .bind(&client, &new_name, &role_arn, &comment, &name)
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
    let aws_iam_role: AwsIamRolePayload = serde_json::from_value(row.credential)?;

    Ok(Credential {
        name: row.name,
        aws_iam_role: AwsIamRole {
            role_arn: aws_iam_role.role_arn,
            external_id: aws_iam_role.external_id,
        },
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

struct CredentialParts {
    name: String,
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
