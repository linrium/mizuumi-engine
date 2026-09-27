use crate::{app_state::AppState, error::AppError};

use super::AuthenticatedPrincipal;
use crate::features::grants::{Privilege, SecurableType};

const METASTORE_NAME: &str = "metastore";

pub async fn can_read_catalog(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Metastore,
            METASTORE_NAME,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }
    has(
        state,
        principal,
        SecurableType::Catalog,
        catalog,
        &[Privilege::Owner, Privilege::UseCatalog],
    )
    .await
}

pub async fn can_read_schema(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
    schema: &str,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Metastore,
            METASTORE_NAME,
            &[Privilege::Owner],
        )
        .await?
        || has(
            state,
            principal,
            SecurableType::Catalog,
            catalog,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }
    if !has(
        state,
        principal,
        SecurableType::Catalog,
        catalog,
        &[Privilege::UseCatalog],
    )
    .await?
    {
        return Ok(false);
    }
    has(
        state,
        principal,
        SecurableType::Schema,
        &format!("{catalog}.{schema}"),
        &[Privilege::Owner, Privilege::UseSchema],
    )
    .await
}

pub async fn can_read_table(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
    schema: &str,
    table: &str,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Metastore,
            METASTORE_NAME,
            &[Privilege::Owner],
        )
        .await?
        || has(
            state,
            principal,
            SecurableType::Catalog,
            catalog,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }
    let catalog_name = catalog;
    let schema_name = format!("{catalog}.{schema}");
    let table_name = format!("{catalog}.{schema}.{table}");
    if !has(
        state,
        principal,
        SecurableType::Catalog,
        catalog_name,
        &[Privilege::UseCatalog],
    )
    .await?
    {
        return Ok(false);
    }
    if has(
        state,
        principal,
        SecurableType::Schema,
        &schema_name,
        &[Privilege::Owner],
    )
    .await?
    {
        return Ok(true);
    }
    if !has(
        state,
        principal,
        SecurableType::Schema,
        &schema_name,
        &[Privilege::UseSchema],
    )
    .await?
    {
        return Ok(false);
    }
    has(
        state,
        principal,
        SecurableType::Table,
        &table_name,
        &[Privilege::Owner, Privilege::Select, Privilege::Modify],
    )
    .await
}

pub async fn can_delete_catalog(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
) -> Result<bool, AppError> {
    if principal.is_admin {
        return Ok(true);
    }
    Ok(has(
        state,
        principal,
        SecurableType::Metastore,
        METASTORE_NAME,
        &[Privilege::Owner],
    )
    .await?
        || has(
            state,
            principal,
            SecurableType::Catalog,
            catalog,
            &[Privilege::Owner],
        )
        .await?)
}

pub async fn can_delete_schema(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
    schema: &str,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Catalog,
            catalog,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }
    Ok(has(
        state,
        principal,
        SecurableType::Catalog,
        catalog,
        &[Privilege::UseCatalog],
    )
    .await?
        && has(
            state,
            principal,
            SecurableType::Schema,
            &format!("{catalog}.{schema}"),
            &[Privilege::Owner],
        )
        .await?)
}

pub async fn can_delete_table(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    catalog: &str,
    schema: &str,
    table: &str,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Catalog,
            catalog,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }
    if !has(
        state,
        principal,
        SecurableType::Catalog,
        catalog,
        &[Privilege::UseCatalog],
    )
    .await?
    {
        return Ok(false);
    }
    let schema_name = format!("{catalog}.{schema}");
    if has(
        state,
        principal,
        SecurableType::Schema,
        &schema_name,
        &[Privilege::Owner],
    )
    .await?
    {
        return Ok(true);
    }
    if !has(
        state,
        principal,
        SecurableType::Schema,
        &schema_name,
        &[Privilege::UseSchema],
    )
    .await?
    {
        return Ok(false);
    }
    has(
        state,
        principal,
        SecurableType::Table,
        &format!("{catalog}.{schema}.{table}"),
        &[Privilege::Owner],
    )
    .await
}

pub async fn require_allowed(allowed: bool, action: &str) -> Result<(), AppError> {
    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!("not authorized to {action}")))
    }
}

pub async fn can_vend_table_credentials(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    table_id: &str,
    read_write: bool,
) -> Result<bool, AppError> {
    if principal.is_admin {
        return Ok(true);
    }
    state
        .grants
        .has_table_access_by_id(principal.id.clone(), table_id.to_owned(), read_write)
        .await
}

pub async fn can_vend_path_credentials(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    path: &str,
    read_write: bool,
) -> Result<bool, AppError> {
    if principal.is_admin
        || has(
            state,
            principal,
            SecurableType::Metastore,
            METASTORE_NAME,
            &[Privilege::Owner],
        )
        .await?
    {
        return Ok(true);
    }

    let normalized_path =
        crate::features::external_locations::normalize_external_location_url(path)?;
    state
        .grants
        .has_external_location_access_by_path(principal.id.clone(), normalized_path, read_write)
        .await
}

async fn has(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    securable_type: SecurableType,
    full_name: &str,
    privileges: &[Privilege],
) -> Result<bool, AppError> {
    if principal.is_admin {
        return Ok(true);
    }
    state
        .grants
        .has_any_privilege(
            principal.id.clone(),
            securable_type,
            full_name.to_owned(),
            privileges.to_vec(),
        )
        .await
}
