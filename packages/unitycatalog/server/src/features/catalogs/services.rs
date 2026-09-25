use std::collections::BTreeMap;

use async_trait::async_trait;
use deadpool_postgres::Pool;
use serde_json::Value;
use tokio_postgres::error::SqlState;
use unitycatalog_queries::queries::catalogs as queries;

use crate::error::AppError;

use super::{
    dtos::{
        CatalogInfo, CreateCatalogRequest, DeleteCatalogRequest, ListCatalogsRequest,
        ListCatalogsResponse, UpdateCatalogRequest,
    },
    models::Catalog,
};

const DEFAULT_PAGE_SIZE: i32 = 100;
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";

#[async_trait]
pub trait CatalogService: Send + Sync {
    async fn create_catalog(&self, request: CreateCatalogRequest) -> Result<CatalogInfo, AppError>;
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
    ) -> Result<ListCatalogsResponse, AppError>;
    async fn get_catalog(&self, name: String) -> Result<CatalogInfo, AppError>;
    async fn update_catalog(
        &self,
        name: String,
        request: UpdateCatalogRequest,
    ) -> Result<CatalogInfo, AppError>;
    async fn delete_catalog(
        &self,
        name: String,
        request: DeleteCatalogRequest,
    ) -> Result<(), AppError>;
}

pub struct DefaultCatalogService {
    pool: Pool,
}

impl DefaultCatalogService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CatalogService for DefaultCatalogService {
    async fn create_catalog(&self, request: CreateCatalogRequest) -> Result<CatalogInfo, AppError> {
        let client = self.pool.get().await?;
        let properties = serde_json::to_value(request.properties.unwrap_or_default())?;
        let comment = request.comment.unwrap_or_default();
        let storage_root = request.storage_root.unwrap_or_default();
        validate_storage_root(&storage_root)?;

        let row = queries::create_catalog()
            .bind(&client, &request.name, &comment, &properties, &storage_root)
            .one()
            .await
            .map_err(map_catalog_write_error)?;

        Ok(row_to_catalog(row)?.into())
    }

    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
    ) -> Result<ListCatalogsResponse, AppError> {
        let page_size = page_size(request.max_results)?;
        let limit = i64::from(page_size);
        let page_token = request.page_token.unwrap_or_default();
        let client = self.pool.get().await?;

        let catalogs = queries::list_catalogs()
            .bind(&client, &page_token, &limit)
            .all()
            .await?
            .into_iter()
            .map(row_to_catalog)
            .collect::<Result<Vec<_>, _>>()?;

        let next_page_token = if catalogs.len() == page_size as usize {
            catalogs.last().map(|catalog| catalog.name.clone())
        } else {
            None
        };

        Ok(ListCatalogsResponse {
            catalogs: catalogs.into_iter().map(Into::into).collect(),
            next_page_token,
        })
    }

    async fn get_catalog(&self, name: String) -> Result<CatalogInfo, AppError> {
        ensure_name(&name)?;
        let client = self.pool.get().await?;
        let row = queries::get_catalog()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("catalog {name}")))?;

        Ok(row_to_catalog(row)?.into())
    }

    async fn update_catalog(
        &self,
        name: String,
        request: UpdateCatalogRequest,
    ) -> Result<CatalogInfo, AppError> {
        ensure_name(&name)?;
        let client = self.pool.get().await?;
        let current = queries::get_catalog()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("catalog {name}")))?;
        let current = row_to_catalog(current)?;

        if request.new_name.is_none()
            && request.comment.is_none()
            && request.properties.as_ref().is_none_or(BTreeMap::is_empty)
        {
            return Ok(current.into());
        }

        let new_name = request.new_name.unwrap_or_else(|| current.name.clone());
        let comment = request.comment.unwrap_or(current.comment);
        let properties = match request.properties {
            Some(properties) if !properties.is_empty() => serde_json::to_value(properties)?,
            None => serde_json::to_value(current.properties)?,
            Some(_) => json_properties(current.properties)?,
        };

        let row = queries::update_catalog()
            .bind(&client, &new_name, &comment, &properties, &name)
            .one()
            .await
            .map_err(map_catalog_write_error)?;

        Ok(row_to_catalog(row)?.into())
    }

    async fn delete_catalog(
        &self,
        name: String,
        request: DeleteCatalogRequest,
    ) -> Result<(), AppError> {
        ensure_name(&name)?;
        let _force = request.force.unwrap_or(false);
        let client = self.pool.get().await?;
        queries::delete_catalog()
            .bind(&client, &name)
            .opt()
            .await?
            .ok_or_else(|| AppError::NotFound(format!("catalog {name}")))?;

        Ok(())
    }
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

fn json_properties(properties: BTreeMap<String, String>) -> Result<Value, AppError> {
    serde_json::to_value(properties).map_err(AppError::from)
}

fn ensure_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::InvalidParameter(
            "catalog name must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_storage_root(storage_root: &str) -> Result<(), AppError> {
    if storage_root.contains(MANAGED_STORAGE_PREFIX) {
        return Err(AppError::InvalidParameter(format!(
            "input path '{storage_root}' contains managed storage prefix {MANAGED_STORAGE_PREFIX}"
        )));
    }

    Ok(())
}

fn map_catalog_write_error(error: tokio_postgres::Error) -> AppError {
    if error.code() == Some(&SqlState::UNIQUE_VIOLATION) {
        AppError::Conflict("catalog name".to_string())
    } else {
        AppError::Postgres(error)
    }
}

fn row_to_catalog(row: impl IntoCatalogParts) -> Result<Catalog, AppError> {
    let row = row.into_catalog_parts();
    let properties = serde_json::from_value(row.properties)?;

    Ok(Catalog {
        name: row.name,
        comment: row.comment,
        properties,
        owner: row.owner,
        created_at: row.created_at,
        created_by: row.created_by,
        updated_at: (row.updated_at > 0).then_some(row.updated_at),
        updated_by: row.updated_by,
        id: row.id,
        storage_root: row.storage_root,
        storage_location: row.storage_location,
    })
}

struct CatalogParts {
    name: String,
    comment: String,
    properties: Value,
    owner: String,
    created_at: i64,
    created_by: String,
    updated_at: i64,
    updated_by: String,
    id: String,
    storage_root: String,
    storage_location: String,
}

trait IntoCatalogParts {
    fn into_catalog_parts(self) -> CatalogParts;
}

macro_rules! impl_into_catalog_parts {
    ($type:ty) => {
        impl IntoCatalogParts for $type {
            fn into_catalog_parts(self) -> CatalogParts {
                CatalogParts {
                    name: self.name,
                    comment: self.comment,
                    properties: self.properties,
                    owner: self.owner,
                    created_at: self.created_at,
                    created_by: self.created_by,
                    updated_at: self.updated_at,
                    updated_by: self.updated_by,
                    id: self.id,
                    storage_root: self.storage_root,
                    storage_location: self.storage_location,
                }
            }
        }
    };
}

impl_into_catalog_parts!(queries::CreateCatalog);
impl_into_catalog_parts!(queries::ListCatalogs);
impl_into_catalog_parts!(queries::GetCatalog);
impl_into_catalog_parts!(queries::UpdateCatalog);
