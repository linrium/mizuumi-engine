// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct GetGrantResourceIdParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub full_name: T1,
    pub securable_type: T2,
}
#[derive(Debug)]
pub struct GrantPermissionParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub principal: T1,
    pub resource_id: T2,
    pub securable_type: T3,
    pub privilege: T4,
}
#[derive(Debug)]
pub struct RevokePermissionParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
{
    pub principal: T1,
    pub resource_id: T2,
    pub privilege: T3,
}
#[derive(Debug)]
pub struct ListPermissionsParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub resource_id: T1,
    pub principal: T2,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ListPermissions {
    pub principal: String,
    pub privilege: String,
}
pub struct ListPermissionsBorrowed<'a> {
    pub principal: &'a str,
    pub privilege: &'a str,
}
impl<'a> From<ListPermissionsBorrowed<'a>> for ListPermissions {
    fn from(
        ListPermissionsBorrowed {
            principal,
            privilege,
        }: ListPermissionsBorrowed<'a>,
    ) -> Self {
        Self {
            principal: principal.into(),
            privilege: privilege.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct StringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&str, tokio_postgres::Error>,
    mapper: fn(&str) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&str) -> R) -> StringQuery<'c, 'a, 's, C, R, N> {
        StringQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct ListPermissionsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ListPermissionsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPermissionsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPermissionsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPermissionsBorrowed) -> R,
    ) -> ListPermissionsQuery<'c, 'a, 's, C, R, N> {
        ListPermissionsQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct GetGrantResourceIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_grant_resource_id() -> GetGrantResourceIdStmt {
    GetGrantResourceIdStmt(
        "SELECT resource.id FROM ( SELECT id, 'metastore'::text AS securable_type, $1::text AS full_name FROM uc_metastore UNION ALL SELECT id, 'catalog', name FROM uc_catalogs UNION ALL SELECT schemas.id, 'schema', CONCAT(catalogs.name, '.', schemas.name) FROM uc_schemas schemas JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id UNION ALL SELECT tables.id, 'table', CONCAT(catalogs.name, '.', schemas.name, '.', tables.name) FROM uc_tables tables JOIN uc_schemas schemas ON schemas.id = tables.schema_id JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id UNION ALL SELECT id, 'credential', name FROM uc_credentials UNION ALL SELECT id, 'external_location', name FROM uc_external_locations ) resource WHERE resource.securable_type = $2 AND resource.full_name = $1",
        None,
    )
}
impl GetGrantResourceIdStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        full_name: &'a T1,
        securable_type: &'a T2,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [full_name, securable_type],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetGrantResourceIdParams<T1, T2>,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for GetGrantResourceIdStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetGrantResourceIdParams<T1, T2>,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.full_name, &params.securable_type)
    }
}
pub struct GrantPermissionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn grant_permission() -> GrantPermissionStmt {
    GrantPermissionStmt(
        "INSERT INTO uc_permissions (principal, resource_id, securable_type, privilege) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING RETURNING privilege",
        None,
    )
}
impl GrantPermissionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        principal: &'a T1,
        resource_id: &'a T2,
        securable_type: &'a T3,
        privilege: &'a T4,
    ) -> StringQuery<'c, 'a, 's, C, String, 4> {
        StringQuery {
            client,
            params: [principal, resource_id, securable_type, privilege],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GrantPermissionParams<T1, T2, T3, T4>,
        StringQuery<'c, 'a, 's, C, String, 4>,
        C,
    > for GrantPermissionStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GrantPermissionParams<T1, T2, T3, T4>,
    ) -> StringQuery<'c, 'a, 's, C, String, 4> {
        self.bind(
            client,
            &params.principal,
            &params.resource_id,
            &params.securable_type,
            &params.privilege,
        )
    }
}
pub struct RevokePermissionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_permission() -> RevokePermissionStmt {
    RevokePermissionStmt(
        "DELETE FROM uc_permissions WHERE principal = $1 AND resource_id = $2 AND privilege = $3 RETURNING privilege",
        None,
    )
}
impl RevokePermissionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        principal: &'a T1,
        resource_id: &'a T2,
        privilege: &'a T3,
    ) -> StringQuery<'c, 'a, 's, C, String, 3> {
        StringQuery {
            client,
            params: [principal, resource_id, privilege],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        RevokePermissionParams<T1, T2, T3>,
        StringQuery<'c, 'a, 's, C, String, 3>,
        C,
    > for RevokePermissionStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a RevokePermissionParams<T1, T2, T3>,
    ) -> StringQuery<'c, 'a, 's, C, String, 3> {
        self.bind(
            client,
            &params.principal,
            &params.resource_id,
            &params.privilege,
        )
    }
}
pub struct ListPermissionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_permissions() -> ListPermissionsStmt {
    ListPermissionsStmt(
        "SELECT principal, privilege FROM uc_permissions WHERE resource_id = $1 AND ($2 = '' OR principal = $2) ORDER BY principal, privilege",
        None,
    )
}
impl ListPermissionsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        resource_id: &'a T1,
        principal: &'a T2,
    ) -> ListPermissionsQuery<'c, 'a, 's, C, ListPermissions, 2> {
        ListPermissionsQuery {
            client,
            params: [resource_id, principal],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListPermissionsBorrowed, tokio_postgres::Error> {
                Ok(ListPermissionsBorrowed {
                    principal: row.try_get(0)?,
                    privilege: row.try_get(1)?,
                })
            },
            mapper: |it| ListPermissions::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        ListPermissionsParams<T1, T2>,
        ListPermissionsQuery<'c, 'a, 's, C, ListPermissions, 2>,
        C,
    > for ListPermissionsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ListPermissionsParams<T1, T2>,
    ) -> ListPermissionsQuery<'c, 'a, 's, C, ListPermissions, 2> {
        self.bind(client, &params.resource_id, &params.principal)
    }
}
