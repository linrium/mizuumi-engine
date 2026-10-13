use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use deadpool_postgres::Pool;
use unitycatalog_queries::queries::grants as queries;

use crate::error::AppError;

use super::dtos::{
    AllAssignmentsList, GetPermissionsRequest, PermissionsList, Privilege, PrivilegeAssignment,
    SecurableAssignments, SecurableType, UpdatePermissions,
};

#[async_trait]
pub trait GrantService: Send + Sync {
    async fn has_any_privilege(
        &self,
        principal: String,
        securable_type: SecurableType,
        full_name: String,
        privileges: Vec<Privilege>,
    ) -> Result<bool, AppError>;
    async fn has_table_access_by_id(
        &self,
        principal: String,
        table_id: String,
        read_write: bool,
    ) -> Result<bool, AppError>;
    async fn has_external_location_access_by_path(
        &self,
        principal: String,
        path: String,
        read_write: bool,
    ) -> Result<bool, AppError>;
    async fn get_permissions(
        &self,
        securable_type: SecurableType,
        full_name: String,
        request: GetPermissionsRequest,
    ) -> Result<PermissionsList, AppError>;

    async fn update_permissions(
        &self,
        securable_type: SecurableType,
        full_name: String,
        request: UpdatePermissions,
    ) -> Result<PermissionsList, AppError>;

    async fn list_all_assignments(&self) -> Result<AllAssignmentsList, AppError>;
}

pub struct DefaultGrantService {
    pool: Pool,
}

impl DefaultGrantService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GrantService for DefaultGrantService {
    async fn has_table_access_by_id(
        &self,
        principal: String,
        table_id: String,
        read_write: bool,
    ) -> Result<bool, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT EXISTS (\
                   SELECT 1 \
                   FROM uc_tables t \
                   JOIN uc_schemas s ON s.id = t.schema_id \
                   JOIN uc_catalogs c ON c.id = s.catalog_id \
                   WHERE t.id = $2 \
                     AND (\
                       EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = c.id AND p.privilege IN ('OWNER', 'USE CATALOG') AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))) \
                     ) \
                     AND EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = s.id AND p.privilege IN ('OWNER', 'USE SCHEMA') AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))) \
                     AND (\
                       EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = t.id AND p.privilege = 'OWNER' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))) \
                       OR (NOT $3 AND EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = t.id AND p.privilege = 'SELECT' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint)))) \
                       OR ($3 AND EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = t.id AND p.privilege = 'SELECT' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))) AND EXISTS (SELECT 1 FROM uc_permissions p WHERE p.principal = $1 AND p.resource_id = t.id AND p.privilege = 'MODIFY' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint)))) \
                     ) \
                 ) AS allowed",
                &[&principal, &table_id, &read_write],
            )
            .await?;
        Ok(row.get("allowed"))
    }

    async fn has_external_location_access_by_path(
        &self,
        principal: String,
        path: String,
        read_write: bool,
    ) -> Result<bool, AppError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "WITH location AS (\
                   SELECT id FROM uc_external_locations \
                   WHERE url = $2 \
                     OR starts_with($2, url || CASE WHEN right(url, 1) = '/' THEN '' ELSE '/' END) \
                   ORDER BY length(url) DESC \
                   LIMIT 1\
                 ) \
                 SELECT EXISTS (\
                   SELECT 1 FROM location l \
                   WHERE EXISTS (\
                     SELECT 1 FROM uc_permissions p \
                     WHERE p.principal = $1 AND p.resource_id = l.id AND p.privilege = 'OWNER' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))\
                   ) OR (\
                     EXISTS (\
                       SELECT 1 FROM uc_permissions p \
                       WHERE p.principal = $1 AND p.resource_id = l.id AND p.privilege = 'READ FILES' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))\
                     ) AND (NOT $3 OR EXISTS (\
                       SELECT 1 FROM uc_permissions p \
                       WHERE p.principal = $1 AND p.resource_id = l.id AND p.privilege = 'WRITE FILES' AND (p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint))\
                     ))\
                   )\
                 ) AS allowed",
                &[&principal, &path, &read_write],
            )
            .await?;
        Ok(row.get("allowed"))
    }

    async fn has_any_privilege(
        &self,
        principal: String,
        securable_type: SecurableType,
        full_name: String,
        privileges: Vec<Privilege>,
    ) -> Result<bool, AppError> {
        let client = self.pool.get().await?;
        let resource_id = get_resource_id(&client, securable_type, &full_name).await?;
        let rows = queries::list_permissions()
            .bind(&client, &resource_id, &principal)
            .all()
            .await?;
        Ok(rows.iter().any(|row| {
            privileges
                .iter()
                .any(|privilege| row.privilege == privilege.as_str())
        }))
    }

    async fn get_permissions(
        &self,
        securable_type: SecurableType,
        full_name: String,
        request: GetPermissionsRequest,
    ) -> Result<PermissionsList, AppError> {
        let client = self.pool.get().await?;
        let resource_id = get_resource_id(&client, securable_type, &full_name).await?;
        let principal = request.principal.unwrap_or_default();
        let rows = queries::list_permissions()
            .bind(&client, &resource_id, &principal)
            .all()
            .await?;

        rows_to_permissions(rows)
    }

    async fn update_permissions(
        &self,
        securable_type: SecurableType,
        full_name: String,
        request: UpdatePermissions,
    ) -> Result<PermissionsList, AppError> {
        validate_browse_scope(securable_type, &request)?;

        let mut client = self.pool.get().await?;
        let resource_id = get_resource_id(&client, securable_type, &full_name).await?;
        let transaction = client.transaction().await?;
        let mut changed_principals = BTreeSet::new();

        for change in request.changes {
            changed_principals.insert(change.principal.clone());
            let expires_at = change.expires_at.unwrap_or_default();
            for privilege in change.add {
                queries::grant_permission()
                    .bind(
                        &transaction,
                        &change.principal,
                        &resource_id,
                        &securable_type.as_str(),
                        &privilege.as_str(),
                        &expires_at,
                    )
                    .opt()
                    .await?;
            }
            for privilege in change.remove {
                queries::revoke_permission()
                    .bind(
                        &transaction,
                        &change.principal,
                        &resource_id,
                        &privilege.as_str(),
                    )
                    .opt()
                    .await?;
            }
        }

        let rows = queries::list_permissions()
            .bind(&transaction, &resource_id, &String::new())
            .all()
            .await?;
        transaction.commit().await?;

        rows_to_permissions(
            rows.into_iter()
                .filter(|row| changed_principals.contains(&row.principal))
                .collect(),
        )
    }

    async fn list_all_assignments(&self) -> Result<AllAssignmentsList, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "WITH resource AS ( \
                   SELECT id, 'catalog'::text AS securable_type, name AS full_name FROM uc_catalogs \
                   UNION ALL \
                   SELECT s.id, 'schema', CONCAT(c.name, '.', s.name) \
                   FROM uc_schemas s JOIN uc_catalogs c ON c.id = s.catalog_id \
                   UNION ALL \
                   SELECT t.id, 'table', CONCAT(c.name, '.', s.name, '.', t.name) \
                   FROM uc_tables t \
                   JOIN uc_schemas s ON s.id = t.schema_id \
                   JOIN uc_catalogs c ON c.id = s.catalog_id \
                   UNION ALL \
                   SELECT id, 'credential', name FROM uc_credentials \
                   UNION ALL \
                   SELECT id, 'external_location', name FROM uc_external_locations \
                 ) \
                 SELECT r.securable_type, r.full_name, p.principal, p.privilege \
                 FROM uc_permissions p \
                 JOIN resource r ON r.id = p.resource_id \
                 WHERE p.expires_at IS NULL OR p.expires_at > ((EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::bigint) \
                 ORDER BY r.securable_type, r.full_name, p.principal, p.privilege",
                &[],
            )
            .await?;

        let mut groups: BTreeMap<(String, String), BTreeMap<String, Vec<Privilege>>> =
            BTreeMap::new();
        for row in rows {
            let securable_type: String = row.get("securable_type");
            let full_name: String = row.get("full_name");
            let principal: String = row.get("principal");
            let privilege: String = row.get("privilege");
            let privilege = Privilege::try_from(privilege).map_err(|privilege| {
                AppError::InvalidParameter(format!("unknown stored privilege: {privilege}"))
            })?;
            groups
                .entry((securable_type, full_name))
                .or_default()
                .entry(principal)
                .or_default()
                .push(privilege);
        }

        let assignments = groups
            .into_iter()
            .map(
                |((securable_type, full_name), by_principal)| SecurableAssignments {
                    securable_type,
                    full_name,
                    privilege_assignments: by_principal
                        .into_iter()
                        .map(|(principal, privileges)| PrivilegeAssignment {
                            principal,
                            privileges,
                        })
                        .collect(),
                },
            )
            .collect();

        Ok(AllAssignmentsList { assignments })
    }
}

async fn get_resource_id(
    client: &deadpool_postgres::Client,
    securable_type: SecurableType,
    full_name: &str,
) -> Result<String, AppError> {
    queries::get_grant_resource_id()
        .bind(client, &full_name, &securable_type.as_str())
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("{} {full_name}", securable_type.as_str())))
}

fn rows_to_permissions(rows: Vec<queries::ListPermissions>) -> Result<PermissionsList, AppError> {
    let mut assignments = BTreeMap::<String, Vec<Privilege>>::new();
    for row in rows {
        let privilege = Privilege::try_from(row.privilege).map_err(|privilege| {
            AppError::InvalidParameter(format!("unknown stored privilege: {privilege}"))
        })?;
        assignments
            .entry(row.principal)
            .or_default()
            .push(privilege);
    }

    Ok(PermissionsList {
        privilege_assignments: assignments
            .into_iter()
            .map(|(principal, privileges)| PrivilegeAssignment {
                principal,
                privileges,
            })
            .collect(),
    })
}

fn validate_browse_scope(
    securable_type: SecurableType,
    request: &UpdatePermissions,
) -> Result<(), AppError> {
    let supports_browse = matches!(
        securable_type,
        SecurableType::Catalog | SecurableType::Schema
    );
    let adds_browse = request
        .changes
        .iter()
        .any(|change| change.add.contains(&Privilege::Browse));

    if adds_browse && !supports_browse {
        return Err(AppError::InvalidParameter(
            "BROWSE can only be granted on catalogs and schemas".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::grants::dtos::PermissionsChange;

    #[test]
    fn groups_permissions_by_principal() {
        let permissions = rows_to_permissions(vec![
            row("writer@example.com", "USE CATALOG"),
            row("reader@example.com", "SELECT"),
            row("reader@example.com", "MODIFY"),
        ])
        .unwrap();

        assert_eq!(permissions.privilege_assignments.len(), 2);
        assert_eq!(
            permissions.privilege_assignments[0].principal,
            "reader@example.com"
        );
        assert_eq!(
            permissions.privilege_assignments[0].privileges,
            vec![Privilege::Select, Privilege::Modify]
        );
        assert_eq!(
            permissions.privilege_assignments[1].principal,
            "writer@example.com"
        );
        assert_eq!(
            permissions.privilege_assignments[1].privileges,
            vec![Privilege::UseCatalog]
        );
    }

    #[test]
    fn browse_can_only_be_added_to_catalogs_and_schemas() {
        for securable_type in [SecurableType::Catalog, SecurableType::Schema] {
            assert!(
                validate_browse_scope(securable_type, &update(vec![Privilege::Browse])).is_ok()
            );
        }

        assert!(matches!(
            validate_browse_scope(SecurableType::Table, &update(vec![Privilege::Browse])),
            Err(AppError::InvalidParameter(_))
        ));
        assert!(
            validate_browse_scope(SecurableType::Table, &update(vec![Privilege::Select])).is_ok()
        );
    }

    #[test]
    fn legacy_table_browse_can_be_removed() {
        let request = UpdatePermissions {
            changes: vec![PermissionsChange {
                principal: "reader@example.com".to_string(),
                add: vec![],
                remove: vec![Privilege::Browse],
                expires_at: None,
            }],
        };

        assert!(validate_browse_scope(SecurableType::Table, &request).is_ok());
    }

    fn update(add: Vec<Privilege>) -> UpdatePermissions {
        UpdatePermissions {
            changes: vec![PermissionsChange {
                principal: "reader@example.com".to_string(),
                add,
                remove: vec![],
                expires_at: None,
            }],
        }
    }

    fn row(principal: &str, privilege: &str) -> queries::ListPermissions {
        queries::ListPermissions {
            principal: principal.to_string(),
            privilege: privilege.to_string(),
        }
    }
}
