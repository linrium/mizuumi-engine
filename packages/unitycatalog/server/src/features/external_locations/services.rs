use std::path::Path;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::{
    credentials as credential_queries, external_locations as queries,
};
use url::Url;

use crate::error::AppError;

use super::{
    dtos::{
        CreateExternalLocationRequest, DeleteExternalLocationRequest, ExternalLocationInfo,
        ListExternalLocationsRequest, ListExternalLocationsResponse, UpdateExternalLocationRequest,
    },
    models::ExternalLocation,
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";
const STORAGE_PURPOSE: &str = "STORAGE";

#[async_trait]
pub trait ExternalLocationService: Send + Sync {
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo, AppError>;
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
    ) -> Result<ListExternalLocationsResponse, AppError>;
    async fn get_external_location(&self, name: String) -> Result<ExternalLocationInfo, AppError>;
    async fn update_external_location(
        &self,
        name: String,
        request: UpdateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo, AppError>;
    async fn delete_external_location(
        &self,
        name: String,
        request: DeleteExternalLocationRequest,
    ) -> Result<(), AppError>;
}

pub struct DefaultExternalLocationService {
    pool: Pool,
}

impl DefaultExternalLocationService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExternalLocationService for DefaultExternalLocationService {
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo, AppError> {
        let client = self.pool.get().await?;
        ensure_storage_credential(&client, &request.credential_name).await?;
        let url = normalize_external_location_url(&request.url)?;
        ensure_url_available(&client, &url, "").await?;
        let comment = request.comment.unwrap_or_default();

        let row = queries::create_external_location()
            .bind(
                &client,
                &request.name,
                &url,
                &comment,
                &request.credential_name,
            )
            .opt()
            .await
            .map_err(map_external_location_write_error)?
            .ok_or_else(|| AppError::NotFound(format!("credential {}", request.credential_name)))?;

        Ok(row_to_external_location(row).into())
    }

    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
    ) -> Result<ListExternalLocationsResponse, AppError> {
        let page_size = page_size(request.max_results)?;
        let limit = i64::from(page_size);
        let page_token = request.page_token.unwrap_or_default();
        let client = self.pool.get().await?;
        let locations = queries::list_external_locations()
            .bind(&client, &page_token, &limit)
            .all()
            .await?
            .into_iter()
            .map(row_to_external_location)
            .collect::<Vec<_>>();
        let next_page_token = if locations.len() == page_size as usize {
            locations.last().map(|location| location.name.clone())
        } else {
            None
        };

        Ok(ListExternalLocationsResponse {
            external_locations: locations.into_iter().map(Into::into).collect(),
            next_page_token,
        })
    }

    async fn get_external_location(&self, name: String) -> Result<ExternalLocationInfo, AppError> {
        let client = self.pool.get().await?;
        let row = queries::get_external_location()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("external location {name}")))?;

        Ok(row_to_external_location(row).into())
    }

    async fn update_external_location(
        &self,
        name: String,
        request: UpdateExternalLocationRequest,
    ) -> Result<ExternalLocationInfo, AppError> {
        let client = self.pool.get().await?;
        let current = queries::get_external_location()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("external location {name}")))?;
        let current = row_to_external_location(current);

        let new_name = request.new_name.unwrap_or(current.name);
        let url = match request.url {
            Some(url) => normalize_external_location_url(&url)?,
            None => current.url,
        };
        ensure_url_available(&client, &url, &current.id).await?;
        let credential_name = request.credential_name.unwrap_or(current.credential_name);
        ensure_storage_credential(&client, &credential_name).await?;
        let comment = request.comment.unwrap_or(current.comment);
        let _owner = request.owner;

        let row = queries::update_external_location()
            .bind(&client, &new_name, &url, &comment, &name, &credential_name)
            .opt()
            .await
            .map_err(map_external_location_write_error)?
            .ok_or_else(|| AppError::NotFound(format!("external location {name}")))?;

        Ok(row_to_external_location(row).into())
    }

    async fn delete_external_location(
        &self,
        name: String,
        request: DeleteExternalLocationRequest,
    ) -> Result<(), AppError> {
        let client = self.pool.get().await?;
        let location = queries::get_external_location()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("external location {name}")))?;

        if !request.force.unwrap_or(false)
            && let Some(table_id) = queries::find_external_table_using_location()
                .bind(&client, &location.url)
                .opt()
                .await?
        {
            return Err(AppError::InvalidParameter(format!(
                "external location still used by table '{table_id}'"
            )));
        }

        queries::delete_external_location()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("external location {name}")))?;

        Ok(())
    }
}

async fn ensure_storage_credential(
    client: &deadpool_postgres::Client,
    name: &str,
) -> Result<(), AppError> {
    let credential = credential_queries::get_credential()
        .bind(client, &name)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("credential {name}")))?;
    if credential.purpose != STORAGE_PURPOSE {
        return Err(AppError::InvalidParameter(format!(
            "credential not of type storage: {name}"
        )));
    }

    Ok(())
}

async fn ensure_url_available(
    client: &deadpool_postgres::Client,
    url: &str,
    exclude_id: &str,
) -> Result<(), AppError> {
    if queries::find_overlapping_external_location()
        .bind(client, &exclude_id, &url)
        .opt()
        .await?
        .is_some()
    {
        return Err(AppError::InvalidParameter(
            "cannot accept an external location that duplicates or overlaps with an existing external location"
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn normalize_external_location_url(input: &str) -> Result<String, AppError> {
    normalize_storage_url(input, false)
}

pub(crate) fn normalize_managed_table_url(input: &str) -> Result<String, AppError> {
    normalize_storage_url(input, true)
}

fn normalize_storage_url(input: &str, allow_managed_storage: bool) -> Result<String, AppError> {
    if input.trim().is_empty() {
        return Err(AppError::InvalidParameter(
            "path cannot be empty".to_string(),
        ));
    }
    if !input.contains('/') {
        return Err(AppError::InvalidParameter(format!(
            "ambiguous path: {input}"
        )));
    }

    let url = if let Ok(url) = Url::parse(input) {
        url
    } else {
        let path = Path::new(input);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|_| AppError::InvalidParameter(format!("unsupported path: {input}")))?
                .join(path)
        };
        Url::from_file_path(absolute)
            .map_err(|_| AppError::InvalidParameter(format!("unsupported path: {input}")))?
    };

    match url.scheme() {
        "file" => {}
        "s3" | "gs" | "abfs" | "abfss" if url.host_str().is_some() => {}
        scheme => {
            return Err(AppError::InvalidParameter(format!(
                "unsupported URI scheme: {scheme}"
            )));
        }
    }

    let mut normalized = url.to_string();
    while normalized.ends_with('/') && !normalized.ends_with(":///") {
        normalized.pop();
    }
    if !allow_managed_storage && normalized.contains(MANAGED_STORAGE_PREFIX) {
        return Err(AppError::InvalidParameter(format!(
            "input path '{normalized}' contains managed storage prefix {MANAGED_STORAGE_PREFIX}"
        )));
    }

    Ok(normalized)
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

fn map_external_location_write_error(error: tokio_postgres::Error) -> AppError {
    if error.code() != Some(&SqlState::UNIQUE_VIOLATION) {
        return AppError::Postgres(error);
    }

    match error.as_db_error().and_then(|error| error.constraint()) {
        Some("uc_external_locations_url_key") => AppError::InvalidParameter(
            "cannot accept an external location that duplicates or overlaps with an existing external location"
                .to_string(),
        ),
        _ => AppError::Conflict("external location name".to_string()),
    }
}

fn row_to_external_location(row: impl IntoExternalLocationParts) -> ExternalLocation {
    let row = row.into_external_location_parts();
    ExternalLocation {
        name: row.name,
        id: row.id,
        url: row.url,
        credential_name: row.credential_name,
        comment: row.comment,
        owner: row.owner,
        credential_id: row.credential_id,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: (row.updated_at > 0).then_some(row.updated_at),
        updated_by: row.updated_by,
    }
}

struct ExternalLocationParts {
    name: String,
    id: String,
    url: String,
    credential_name: String,
    comment: String,
    owner: String,
    credential_id: String,
    created_at: i64,
    created_by: String,
    updated_at: i64,
    updated_by: String,
}

trait IntoExternalLocationParts {
    fn into_external_location_parts(self) -> ExternalLocationParts;
}

macro_rules! impl_into_external_location_parts {
    ($type:ty) => {
        impl IntoExternalLocationParts for $type {
            fn into_external_location_parts(self) -> ExternalLocationParts {
                ExternalLocationParts {
                    name: self.name,
                    id: self.id,
                    url: self.url,
                    credential_name: self.credential_name,
                    comment: self.comment,
                    owner: self.owner,
                    credential_id: self.credential_id,
                    created_at: self.created_at,
                    created_by: self.created_by,
                    updated_at: self.updated_at,
                    updated_by: self.updated_by,
                }
            }
        }
    };
}

impl_into_external_location_parts!(queries::CreateExternalLocation);
impl_into_external_location_parts!(queries::ListExternalLocations);
impl_into_external_location_parts!(queries::GetExternalLocation);
impl_into_external_location_parts!(queries::UpdateExternalLocation);

#[cfg(test)]
mod tests {
    use super::{normalize_external_location_url, normalize_managed_table_url};

    #[test]
    fn normalizes_supported_urls() {
        assert_eq!(
            normalize_external_location_url("s3://bucket/path/").unwrap(),
            "s3://bucket/path"
        );
        assert_eq!(
            normalize_external_location_url("gs://bucket/a/../path").unwrap(),
            "gs://bucket/path"
        );
        assert_eq!(
            normalize_external_location_url("file:/tmp/path/").unwrap(),
            "file:///tmp/path"
        );
    }

    #[test]
    fn rejects_unsupported_and_managed_urls() {
        assert!(normalize_external_location_url("ftp://host/path").is_err());
        assert!(normalize_external_location_url("s3://bucket/__unitystorage/path").is_err());
        assert_eq!(
            normalize_managed_table_url("s3://bucket/__unitystorage/path/").unwrap(),
            "s3://bucket/__unitystorage/path"
        );
    }
}
