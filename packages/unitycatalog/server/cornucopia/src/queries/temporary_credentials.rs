// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct GetModelVersionStorageLocationParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub model_name: T3,
    pub version: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GetModelVersionStorageLocation {
    pub storage_location: String,
    pub status: String,
}
pub struct GetModelVersionStorageLocationBorrowed<'a> {
    pub storage_location: &'a str,
    pub status: &'a str,
}
impl<'a> From<GetModelVersionStorageLocationBorrowed<'a>> for GetModelVersionStorageLocation {
    fn from(
        GetModelVersionStorageLocationBorrowed {
            storage_location,
            status,
        }: GetModelVersionStorageLocationBorrowed<'a>,
    ) -> Self {
        Self {
            storage_location: storage_location.into(),
            status: status.into(),
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
pub struct GetModelVersionStorageLocationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<GetModelVersionStorageLocationBorrowed, tokio_postgres::Error>,
    mapper: fn(GetModelVersionStorageLocationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetModelVersionStorageLocationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetModelVersionStorageLocationBorrowed) -> R,
    ) -> GetModelVersionStorageLocationQuery<'c, 'a, 's, C, R, N> {
        GetModelVersionStorageLocationQuery {
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
pub struct GetTableStorageLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_table_storage_location() -> GetTableStorageLocationStmt {
    GetTableStorageLocationStmt(
        "SELECT storage_location FROM uc_tables WHERE id = $1::text AND NULLIF(storage_location, '') IS NOT NULL",
        None,
    )
}
impl GetTableStorageLocationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        table_id: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [table_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct GetVolumeStorageLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_volume_storage_location() -> GetVolumeStorageLocationStmt {
    GetVolumeStorageLocationStmt(
        "SELECT storage_location FROM uc_volumes WHERE id = $1::text AND NULLIF(storage_location, '') IS NOT NULL",
        None,
    )
}
impl GetVolumeStorageLocationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        volume_id: &'a T1,
    ) -> StringQuery<'c, 'a, 's, C, String, 1> {
        StringQuery {
            client,
            params: [volume_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
pub struct GetModelVersionStorageLocationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_model_version_storage_location() -> GetModelVersionStorageLocationStmt {
    GetModelVersionStorageLocationStmt(
        "SELECT model_versions.url AS storage_location, model_versions.status FROM uc_model_versions model_versions JOIN uc_registered_models models ON models.id = model_versions.registered_model_id JOIN uc_schemas schemas ON schemas.id = models.schema_id JOIN uc_catalogs catalogs ON catalogs.id = schemas.catalog_id WHERE catalogs.name = $1::text AND schemas.name = $2::text AND models.name = $3::text AND model_versions.version = $4 AND NULLIF(model_versions.url, '') IS NOT NULL",
        None,
    )
}
impl GetModelVersionStorageLocationStmt {
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
        catalog_name: &'a T1,
        schema_name: &'a T2,
        model_name: &'a T3,
        version: &'a i64,
    ) -> GetModelVersionStorageLocationQuery<'c, 'a, 's, C, GetModelVersionStorageLocation, 4> {
        GetModelVersionStorageLocationQuery {
            client,
            params: [catalog_name, schema_name, model_name, version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row: &tokio_postgres::Row| -> Result<
                GetModelVersionStorageLocationBorrowed,
                tokio_postgres::Error,
            > {
                Ok(GetModelVersionStorageLocationBorrowed {
                    storage_location: row.try_get(0)?,
                    status: row.try_get(1)?,
                })
            },
            mapper: |it| GetModelVersionStorageLocation::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetModelVersionStorageLocationParams<T1, T2, T3>,
        GetModelVersionStorageLocationQuery<'c, 'a, 's, C, GetModelVersionStorageLocation, 4>,
        C,
    > for GetModelVersionStorageLocationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetModelVersionStorageLocationParams<T1, T2, T3>,
    ) -> GetModelVersionStorageLocationQuery<'c, 'a, 's, C, GetModelVersionStorageLocation, 4> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.model_name,
            &params.version,
        )
    }
}
