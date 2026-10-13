// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug)]
pub struct FindDataContractParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
{
    pub catalog_name: T1,
    pub schema_name: T2,
    pub table_name: T3,
}
#[derive(Debug)]
pub struct CreateDataContractParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub id: T1,
    pub catalog_name: T2,
    pub schema_name: T3,
    pub table_name: T4,
    pub created_by: T5,
}
#[derive(Debug)]
pub struct GetDataContractVersionParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub contract_id: T1,
    pub version: T2,
}
#[derive(Debug)]
pub struct CreateDataContractVersionParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::JsonSql,
    T4: crate::JsonSql,
    T5: crate::StringSql,
> {
    pub contract_id: T1,
    pub version: T2,
    pub document: T3,
    pub normalized_schema: T4,
    pub created_by: T5,
}
#[derive(Debug)]
pub struct ActivateDataContractVersionParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub version: T1,
    pub contract_id: T2,
}
#[derive(Debug)]
pub struct SetPendingDataContractObservationParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub contract_id: T1,
    pub contract_version: T2,
}
#[derive(Debug)]
pub struct GetActiveDataContractParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub catalog_name: T1,
    pub schema_name: T2,
    pub table_name: T3,
}
#[derive(Debug)]
pub struct UpsertDataContractObservationParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::JsonSql,
    T6: crate::JsonSql,
    T7: crate::JsonSql,
    T8: crate::JsonSql,
> {
    pub contract_id: T1,
    pub contract_version: T2,
    pub unity_table_id: T3,
    pub state: T4,
    pub observed_columns: T5,
    pub observed_properties: T6,
    pub normalized_schema: T7,
    pub differences: T8,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FindDataContract {
    pub id: String,
    pub active_version: String,
}
pub struct FindDataContractBorrowed<'a> {
    pub id: &'a str,
    pub active_version: &'a str,
}
impl<'a> From<FindDataContractBorrowed<'a>> for FindDataContract {
    fn from(FindDataContractBorrowed { id, active_version }: FindDataContractBorrowed<'a>) -> Self {
        Self {
            id: id.into(),
            active_version: active_version.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetDataContractVersion {
    pub document: serde_json::Value,
    pub document_hash: String,
    pub schema_hash: String,
    pub created_by: String,
    pub created_at: String,
}
pub struct GetDataContractVersionBorrowed<'a> {
    pub document: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub document_hash: &'a str,
    pub schema_hash: &'a str,
    pub created_by: &'a str,
    pub created_at: &'a str,
}
impl<'a> From<GetDataContractVersionBorrowed<'a>> for GetDataContractVersion {
    fn from(
        GetDataContractVersionBorrowed {
            document,
            document_hash,
            schema_hash,
            created_by,
            created_at,
        }: GetDataContractVersionBorrowed<'a>,
    ) -> Self {
        Self {
            document: serde_json::from_str(document.0.get()).unwrap(),
            document_hash: document_hash.into(),
            schema_hash: schema_hash.into(),
            created_by: created_by.into(),
            created_at: created_at.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GetActiveDataContract {
    pub id: String,
    pub catalog_name: String,
    pub schema_name: String,
    pub table_name: String,
    pub version: String,
    pub document: serde_json::Value,
    pub document_hash: String,
    pub schema_hash: String,
    pub created_by: String,
    pub created_at: String,
    pub sync_state: String,
    pub unity_table_id: String,
    pub observed_schema_hash: String,
    pub differences: serde_json::Value,
    pub checked_at: String,
}
pub struct GetActiveDataContractBorrowed<'a> {
    pub id: &'a str,
    pub catalog_name: &'a str,
    pub schema_name: &'a str,
    pub table_name: &'a str,
    pub version: &'a str,
    pub document: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub document_hash: &'a str,
    pub schema_hash: &'a str,
    pub created_by: &'a str,
    pub created_at: &'a str,
    pub sync_state: &'a str,
    pub unity_table_id: &'a str,
    pub observed_schema_hash: &'a str,
    pub differences: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub checked_at: &'a str,
}
impl<'a> From<GetActiveDataContractBorrowed<'a>> for GetActiveDataContract {
    fn from(
        GetActiveDataContractBorrowed {
            id,
            catalog_name,
            schema_name,
            table_name,
            version,
            document,
            document_hash,
            schema_hash,
            created_by,
            created_at,
            sync_state,
            unity_table_id,
            observed_schema_hash,
            differences,
            checked_at,
        }: GetActiveDataContractBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            table_name: table_name.into(),
            version: version.into(),
            document: serde_json::from_str(document.0.get()).unwrap(),
            document_hash: document_hash.into(),
            schema_hash: schema_hash.into(),
            created_by: created_by.into(),
            created_at: created_at.into(),
            sync_state: sync_state.into(),
            unity_table_id: unity_table_id.into(),
            observed_schema_hash: observed_schema_hash.into(),
            differences: serde_json::from_str(differences.0.get()).unwrap(),
            checked_at: checked_at.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct FindDataContractQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<FindDataContractBorrowed, tokio_postgres::Error>,
    mapper: fn(FindDataContractBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> FindDataContractQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(FindDataContractBorrowed) -> R,
    ) -> FindDataContractQuery<'c, 'a, 's, C, R, N> {
        FindDataContractQuery {
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
pub struct GetDataContractVersionQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<GetDataContractVersionBorrowed, tokio_postgres::Error>,
    mapper: fn(GetDataContractVersionBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetDataContractVersionQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetDataContractVersionBorrowed) -> R,
    ) -> GetDataContractVersionQuery<'c, 'a, 's, C, R, N> {
        GetDataContractVersionQuery {
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
pub struct GetActiveDataContractQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<GetActiveDataContractBorrowed, tokio_postgres::Error>,
    mapper: fn(GetActiveDataContractBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GetActiveDataContractQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GetActiveDataContractBorrowed) -> R,
    ) -> GetActiveDataContractQuery<'c, 'a, 's, C, R, N> {
        GetActiveDataContractQuery {
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
pub struct FindDataContractStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn find_data_contract() -> FindDataContractStmt {
    FindDataContractStmt(
        "SELECT id, COALESCE(active_version, '') AS active_version FROM data_contracts WHERE catalog_name = $1::text AND schema_name = $2::text AND table_name = $3::text",
        None,
    )
}
impl FindDataContractStmt {
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
        table_name: &'a T3,
    ) -> FindDataContractQuery<'c, 'a, 's, C, FindDataContract, 3> {
        FindDataContractQuery {
            client,
            params: [catalog_name, schema_name, table_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<FindDataContractBorrowed, tokio_postgres::Error> {
                Ok(FindDataContractBorrowed {
                    id: row.try_get(0)?,
                    active_version: row.try_get(1)?,
                })
            },
            mapper: |it| FindDataContract::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        FindDataContractParams<T1, T2, T3>,
        FindDataContractQuery<'c, 'a, 's, C, FindDataContract, 3>,
        C,
    > for FindDataContractStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a FindDataContractParams<T1, T2, T3>,
    ) -> FindDataContractQuery<'c, 'a, 's, C, FindDataContract, 3> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.table_name,
        )
    }
}
pub struct CreateDataContractStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_data_contract() -> CreateDataContractStmt {
    CreateDataContractStmt(
        "INSERT INTO data_contracts ( id, catalog_name, schema_name, table_name, created_by ) VALUES ( $1::text, $2::text, $3::text, $4::text, $5::text )",
        None,
    )
}
impl CreateDataContractStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a T1,
        catalog_name: &'a T2,
        schema_name: &'a T3,
        table_name: &'a T4,
        created_by: &'a T5,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[id, catalog_name, schema_name, table_name, created_by],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateDataContractParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateDataContractStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateDataContractParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.catalog_name,
            &params.schema_name,
            &params.table_name,
            &params.created_by,
        ))
    }
}
pub struct GetDataContractVersionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_data_contract_version() -> GetDataContractVersionStmt {
    GetDataContractVersionStmt(
        "SELECT document, document_hash, schema_hash, created_by, created_at::text AS created_at FROM data_contract_versions WHERE contract_id = $1::text AND version = $2::text",
        None,
    )
}
impl GetDataContractVersionStmt {
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
        contract_id: &'a T1,
        version: &'a T2,
    ) -> GetDataContractVersionQuery<'c, 'a, 's, C, GetDataContractVersion, 2> {
        GetDataContractVersionQuery {
            client,
            params: [contract_id, version],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetDataContractVersionBorrowed, tokio_postgres::Error> {
                Ok(GetDataContractVersionBorrowed {
                    document: row.try_get(0)?,
                    document_hash: row.try_get(1)?,
                    schema_hash: row.try_get(2)?,
                    created_by: row.try_get(3)?,
                    created_at: row.try_get(4)?,
                })
            },
            mapper: |it| GetDataContractVersion::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetDataContractVersionParams<T1, T2>,
        GetDataContractVersionQuery<'c, 'a, 's, C, GetDataContractVersion, 2>,
        C,
    > for GetDataContractVersionStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetDataContractVersionParams<T1, T2>,
    ) -> GetDataContractVersionQuery<'c, 'a, 's, C, GetDataContractVersion, 2> {
        self.bind(client, &params.contract_id, &params.version)
    }
}
pub struct CreateDataContractVersionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn create_data_contract_version() -> CreateDataContractVersionStmt {
    CreateDataContractVersionStmt(
        "INSERT INTO data_contract_versions ( contract_id, version, document, document_hash, schema_hash, created_by ) VALUES ( $1::text, $2::text, $3, encode(digest(($3)::jsonb::text, 'sha256'), 'hex'), encode(digest(($4)::jsonb::text, 'sha256'), 'hex'), $5::text )",
        None,
    )
}
impl CreateDataContractVersionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::JsonSql,
        T4: crate::JsonSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        contract_id: &'a T1,
        version: &'a T2,
        document: &'a T3,
        normalized_schema: &'a T4,
        created_by: &'a T5,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    contract_id,
                    version,
                    document,
                    normalized_schema,
                    created_by,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::JsonSql,
    T4: crate::JsonSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        CreateDataContractVersionParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for CreateDataContractVersionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a CreateDataContractVersionParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.contract_id,
            &params.version,
            &params.document,
            &params.normalized_schema,
            &params.created_by,
        ))
    }
}
pub struct ActivateDataContractVersionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn activate_data_contract_version() -> ActivateDataContractVersionStmt {
    ActivateDataContractVersionStmt(
        "UPDATE data_contracts SET active_version = $1::text, updated_at = NOW() WHERE id = $2::text",
        None,
    )
}
impl ActivateDataContractVersionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        version: &'a T1,
        contract_id: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[version, contract_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        ActivateDataContractVersionParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for ActivateDataContractVersionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a ActivateDataContractVersionParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.version, &params.contract_id))
    }
}
pub struct SetPendingDataContractObservationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_pending_data_contract_observation() -> SetPendingDataContractObservationStmt {
    SetPendingDataContractObservationStmt(
        "INSERT INTO data_contract_observations ( contract_id, contract_version, state ) VALUES ( $1::text, $2::text, 'PENDING_DEPLOYMENT' ) ON CONFLICT (contract_id) DO UPDATE SET contract_version = EXCLUDED.contract_version, unity_table_id = NULL, state = EXCLUDED.state, observed_columns = '[]'::jsonb, observed_properties = '{}'::jsonb, observed_schema_hash = NULL, differences = '[]'::jsonb, checked_at = NOW()",
        None,
    )
}
impl SetPendingDataContractObservationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        contract_id: &'a T1,
        contract_version: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[contract_id, contract_version])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetPendingDataContractObservationParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetPendingDataContractObservationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetPendingDataContractObservationParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.contract_id, &params.contract_version))
    }
}
pub struct GetActiveDataContractStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn get_active_data_contract() -> GetActiveDataContractStmt {
    GetActiveDataContractStmt(
        "SELECT contracts.id, contracts.catalog_name, contracts.schema_name, contracts.table_name, versions.version, versions.document, versions.document_hash, versions.schema_hash, versions.created_by, versions.created_at::text AS created_at, COALESCE(observations.state, 'PENDING_DEPLOYMENT') AS sync_state, COALESCE(observations.unity_table_id, '') AS unity_table_id, COALESCE(observations.observed_schema_hash, '') AS observed_schema_hash, COALESCE(observations.differences, '[]'::jsonb) AS differences, COALESCE(observations.checked_at::text, '') AS checked_at FROM data_contracts contracts JOIN data_contract_versions versions ON versions.contract_id = contracts.id AND versions.version = contracts.active_version LEFT JOIN data_contract_observations observations ON observations.contract_id = contracts.id WHERE contracts.catalog_name = $1::text AND contracts.schema_name = $2::text AND contracts.table_name = $3::text",
        None,
    )
}
impl GetActiveDataContractStmt {
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
        table_name: &'a T3,
    ) -> GetActiveDataContractQuery<'c, 'a, 's, C, GetActiveDataContract, 3> {
        GetActiveDataContractQuery {
            client,
            params: [catalog_name, schema_name, table_name],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<GetActiveDataContractBorrowed, tokio_postgres::Error> {
                Ok(GetActiveDataContractBorrowed {
                    id: row.try_get(0)?,
                    catalog_name: row.try_get(1)?,
                    schema_name: row.try_get(2)?,
                    table_name: row.try_get(3)?,
                    version: row.try_get(4)?,
                    document: row.try_get(5)?,
                    document_hash: row.try_get(6)?,
                    schema_hash: row.try_get(7)?,
                    created_by: row.try_get(8)?,
                    created_at: row.try_get(9)?,
                    sync_state: row.try_get(10)?,
                    unity_table_id: row.try_get(11)?,
                    observed_schema_hash: row.try_get(12)?,
                    differences: row.try_get(13)?,
                    checked_at: row.try_get(14)?,
                })
            },
            mapper: |it| GetActiveDataContract::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        GetActiveDataContractParams<T1, T2, T3>,
        GetActiveDataContractQuery<'c, 'a, 's, C, GetActiveDataContract, 3>,
        C,
    > for GetActiveDataContractStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a GetActiveDataContractParams<T1, T2, T3>,
    ) -> GetActiveDataContractQuery<'c, 'a, 's, C, GetActiveDataContract, 3> {
        self.bind(
            client,
            &params.catalog_name,
            &params.schema_name,
            &params.table_name,
        )
    }
}
pub struct UpsertDataContractObservationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn upsert_data_contract_observation() -> UpsertDataContractObservationStmt {
    UpsertDataContractObservationStmt(
        "INSERT INTO data_contract_observations ( contract_id, contract_version, unity_table_id, state, observed_columns, observed_properties, observed_schema_hash, differences, checked_at ) VALUES ( $1::text, $2::text, NULLIF($3::text, ''), $4::text, $5, $6, CASE WHEN jsonb_typeof(($7)::jsonb) = 'array' THEN encode(digest(($7)::jsonb::text, 'sha256'), 'hex') ELSE NULL END, $8, NOW() ) ON CONFLICT (contract_id) DO UPDATE SET contract_version = EXCLUDED.contract_version, unity_table_id = EXCLUDED.unity_table_id, state = EXCLUDED.state, observed_columns = EXCLUDED.observed_columns, observed_properties = EXCLUDED.observed_properties, observed_schema_hash = EXCLUDED.observed_schema_hash, differences = EXCLUDED.differences, checked_at = NOW()",
        None,
    )
}
impl UpsertDataContractObservationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::JsonSql,
        T6: crate::JsonSql,
        T7: crate::JsonSql,
        T8: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        contract_id: &'a T1,
        contract_version: &'a T2,
        unity_table_id: &'a T3,
        state: &'a T4,
        observed_columns: &'a T5,
        observed_properties: &'a T6,
        normalized_schema: &'a T7,
        differences: &'a T8,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    contract_id,
                    contract_version,
                    unity_table_id,
                    state,
                    observed_columns,
                    observed_properties,
                    normalized_schema,
                    differences,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::JsonSql,
    T6: crate::JsonSql,
    T7: crate::JsonSql,
    T8: crate::JsonSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpsertDataContractObservationParams<T1, T2, T3, T4, T5, T6, T7, T8>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpsertDataContractObservationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpsertDataContractObservationParams<T1, T2, T3, T4, T5, T6, T7, T8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.contract_id,
            &params.contract_version,
            &params.unity_table_id,
            &params.state,
            &params.observed_columns,
            &params.observed_properties,
            &params.normalized_schema,
            &params.differences,
        ))
    }
}
