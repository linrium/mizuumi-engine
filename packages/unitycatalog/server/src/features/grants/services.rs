use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use deadpool_postgres::Pool;
use unitycatalog_queries::queries::grants as queries;

use crate::error::AppError;

use super::dtos::{
    GetPermissionsRequest, PermissionsList, Privilege, PrivilegeAssignment, SecurableType,
    UpdatePermissions,
};

#[async_trait]
pub trait GrantService: Send + Sync {
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
        let mut client = self.pool.get().await?;
        let resource_id = get_resource_id(&client, securable_type, &full_name).await?;
        let transaction = client.transaction().await?;
        let mut changed_principals = BTreeSet::new();

        for change in request.changes {
            changed_principals.insert(change.principal.clone());
            for privilege in change.add {
                queries::grant_permission()
                    .bind(
                        &transaction,
                        &change.principal,
                        &resource_id,
                        &securable_type.as_str(),
                        &privilege.as_str(),
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

#[cfg(test)]
mod tests {
    use super::*;

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

    fn row(principal: &str, privilege: &str) -> queries::ListPermissions {
        queries::ListPermissions {
            principal: principal.to_string(),
            privilege: privilege.to_string(),
        }
    }
}
