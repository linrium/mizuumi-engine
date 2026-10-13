use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use controlplane_queries::queries::data_contracts as queries;
use deadpool_postgres::Pool;
use serde_json::{Value, json};
use tokio_postgres::error::SqlState;

use crate::{
    error::AppError,
    infrastructure::unity_catalog::{UnityCatalog, UnityColumn, UnityTable},
};

use super::dtos::{DataContract, PutDataContractRequest};

const CONTRACT_ID_PROPERTY: &str = "mizuumi.odcs.id";
const CONTRACT_VERSION_PROPERTY: &str = "mizuumi.odcs.version";
const DOCUMENT_HASH_PROPERTY: &str = "mizuumi.odcs.documentHash";
const SCHEMA_HASH_PROPERTY: &str = "mizuumi.odcs.schemaHash";

#[async_trait]
pub trait DataContractService: Send + Sync {
    async fn put_data_contract(
        &self,
        full_name: String,
        version: String,
        request: PutDataContractRequest,
    ) -> Result<DataContract, AppError>;
    async fn get_data_contract(&self, full_name: String) -> Result<DataContract, AppError>;
    async fn reconcile_data_contract(&self, full_name: String) -> Result<DataContract, AppError>;
}

pub struct DefaultDataContractService {
    pool: Pool,
    unity_catalog: Arc<dyn UnityCatalog>,
}

impl DefaultDataContractService {
    pub fn new(pool: Pool, unity_catalog: Arc<dyn UnityCatalog>) -> Self {
        Self {
            pool,
            unity_catalog,
        }
    }
}

#[async_trait]
impl DataContractService for DefaultDataContractService {
    async fn put_data_contract(
        &self,
        full_name: String,
        version: String,
        request: PutDataContractRequest,
    ) -> Result<DataContract, AppError> {
        require_text("created_by", &request.created_by)?;
        let table = TableName::parse(&full_name)?;
        let validated = validate_document(&request.document, &table, &version)?;

        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let existing_contract = queries::find_data_contract()
            .bind(&transaction, &table.catalog, &table.schema, &table.table)
            .opt()
            .await?;
        if let Some(existing) = &existing_contract
            && !existing.active_version.is_empty()
            && existing.active_version != version
            && !version_is_newer(&version, &existing.active_version)
        {
            return Err(AppError::Conflict(format!(
                "data contract version {version} cannot replace active version {}",
                existing.active_version
            )));
        }
        let contract_id = match existing_contract {
            Some(existing) if existing.id != validated.id => {
                return Err(AppError::Conflict(format!(
                    "table {full_name} is already bound to data contract {}",
                    existing.id
                )));
            }
            Some(existing) => existing.id,
            None => {
                queries::create_data_contract()
                    .bind(
                        &transaction,
                        &validated.id,
                        &table.catalog,
                        &table.schema,
                        &table.table,
                        &request.created_by,
                    )
                    .await
                    .map_err(map_write_error)?;
                validated.id.clone()
            }
        };

        match queries::get_data_contract_version()
            .bind(&transaction, &contract_id, &version)
            .opt()
            .await?
        {
            Some(existing) if existing.document != request.document => {
                return Err(AppError::Conflict(format!(
                    "data contract {contract_id} version {version} is immutable"
                )));
            }
            Some(_) => {}
            None => {
                queries::create_data_contract_version()
                    .bind(
                        &transaction,
                        &contract_id,
                        &version,
                        &request.document,
                        &validated.normalized_schema,
                        &request.created_by,
                    )
                    .await
                    .map_err(map_write_error)?;
            }
        }
        queries::activate_data_contract_version()
            .bind(&transaction, &version, &contract_id)
            .await?;
        queries::set_pending_data_contract_observation()
            .bind(&transaction, &contract_id, &version)
            .await?;
        transaction.commit().await?;

        load_active_contract(&client, &table).await
    }

    async fn get_data_contract(&self, full_name: String) -> Result<DataContract, AppError> {
        let table = TableName::parse(&full_name)?;
        let client = self.pool.get().await?;
        load_active_contract(&client, &table).await
    }

    async fn reconcile_data_contract(&self, full_name: String) -> Result<DataContract, AppError> {
        let table_name = TableName::parse(&full_name)?;
        let client = self.pool.get().await?;
        let contract = load_active_contract(&client, &table_name).await?;
        let observed = self
            .unity_catalog
            .get_table(&full_name)
            .await
            .map_err(|error| AppError::UnityCatalog(error.to_string()))?;

        let (table_id, state, columns, properties, normalized_schema, differences) = match observed
        {
            Some(mut table) => {
                let expected_properties = contract_properties(&contract);
                if expected_properties
                    .iter()
                    .any(|(key, value)| table.properties.get(key) != Some(value))
                {
                    table = self
                        .unity_catalog
                        .update_table_properties(&full_name, &expected_properties)
                        .await
                        .map_err(|error| AppError::UnityCatalog(error.to_string()))?;
                }
                let observed_schema = normalize_unity_schema(&table.columns)?;
                let mut differences = compare_schema(&contract.document, &observed_schema)?;
                compare_contract_properties(&contract, &table, &mut differences);
                let state = if differences
                    .as_array()
                    .is_some_and(|items| items.iter().any(is_error_difference))
                {
                    "DRIFTED"
                } else {
                    "IN_SYNC"
                };
                (
                    table.table_id,
                    state,
                    serde_json::to_value(table.columns)?,
                    serde_json::to_value(table.properties)?,
                    observed_schema,
                    differences,
                )
            }
            None => (
                String::new(),
                "TABLE_MISSING",
                json!([]),
                json!({}),
                Value::Null,
                json!([{
                    "severity": "ERROR",
                    "code": "TABLE_MISSING",
                    "path": full_name,
                    "expected": "managed Delta table",
                    "actual": null
                }]),
            ),
        };

        queries::upsert_data_contract_observation()
            .bind(
                &client,
                &contract.id,
                &contract.version,
                &table_id,
                &state,
                &columns,
                &properties,
                &normalized_schema,
                &differences,
            )
            .await?;
        load_active_contract(&client, &table_name).await
    }
}

#[derive(Debug)]
struct TableName {
    catalog: String,
    schema: String,
    table: String,
}

impl TableName {
    fn parse(full_name: &str) -> Result<Self, AppError> {
        let parts: Vec<_> = full_name.split('.').collect();
        if parts.len() != 3 || parts.iter().any(|part| part.trim().is_empty()) {
            return Err(AppError::InvalidParameter(format!(
                "invalid table full name: {full_name}"
            )));
        }
        Ok(Self {
            catalog: parts[0].to_owned(),
            schema: parts[1].to_owned(),
            table: parts[2].to_owned(),
        })
    }

    fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.schema, self.table)
    }
}

struct ValidatedDocument {
    id: String,
    normalized_schema: Value,
}

fn validate_document(
    document: &Value,
    table: &TableName,
    path_version: &str,
) -> Result<ValidatedDocument, AppError> {
    let object = document.as_object().ok_or_else(|| {
        AppError::InvalidParameter("data contract document must be a JSON object".to_owned())
    })?;
    require_json_string(object.get("apiVersion"), "apiVersion", Some("v3.1.0"))?;
    require_json_string(object.get("kind"), "kind", Some("DataContract"))?;
    let id = require_json_string(object.get("id"), "id", None)?.to_owned();
    let version = require_json_string(object.get("version"), "version", None)?;
    if version != path_version {
        return Err(AppError::InvalidParameter(format!(
            "document version {version} does not match URL version {path_version}"
        )));
    }
    if !is_semver(version) {
        return Err(AppError::InvalidParameter(
            "version must be a semantic version such as 1.0.0".to_owned(),
        ));
    }
    require_json_string(object.get("status"), "status", Some("active"))?;
    require_json_string(object.get("name"), "name", None)?;

    let schema = object
        .get("schema")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::InvalidParameter("schema must be an array".to_owned()))?;
    if schema.len() != 1 {
        return Err(AppError::InvalidParameter(
            "table contracts must contain exactly one schema object".to_owned(),
        ));
    }
    let schema_object = schema[0]
        .as_object()
        .ok_or_else(|| AppError::InvalidParameter("schema item must be an object".to_owned()))?;
    require_json_string(
        schema_object.get("logicalType"),
        "schema[0].logicalType",
        Some("object"),
    )?;
    require_json_string(
        schema_object.get("physicalType"),
        "schema[0].physicalType",
        Some("table"),
    )?;
    let physical_name = require_json_string(
        schema_object.get("physicalName"),
        "schema[0].physicalName",
        None,
    )?;
    if physical_name != table.full_name() {
        return Err(AppError::InvalidParameter(format!(
            "schema physicalName must be {}",
            table.full_name()
        )));
    }
    let normalized_schema = normalized_contract_schema(document)?;

    Ok(ValidatedDocument {
        id,
        normalized_schema,
    })
}

fn normalized_contract_schema(document: &Value) -> Result<Value, AppError> {
    let properties = document["schema"][0]["properties"]
        .as_array()
        .ok_or_else(|| {
            AppError::InvalidParameter("schema[0].properties must be an array".to_owned())
        })?;
    if properties.is_empty() {
        return Err(AppError::InvalidParameter(
            "schema[0].properties must not be empty".to_owned(),
        ));
    }
    let mut names = BTreeSet::new();
    let mut normalized = Vec::with_capacity(properties.len());
    for (index, property) in properties.iter().enumerate() {
        let property = property.as_object().ok_or_else(|| {
            AppError::InvalidParameter(format!("schema property {index} must be an object"))
        })?;
        let name = require_json_string(property.get("name"), "property.name", None)?;
        if !names.insert(name.to_owned()) {
            return Err(AppError::InvalidParameter(format!(
                "duplicate schema property: {name}"
            )));
        }
        let logical_type =
            require_json_string(property.get("logicalType"), "property.logicalType", None)?;
        if !matches!(
            logical_type,
            "string" | "date" | "timestamp" | "time" | "number" | "integer" | "boolean"
        ) {
            return Err(AppError::InvalidParameter(format!(
                "unsupported logicalType for property {name}: {logical_type}"
            )));
        }
        let physical_type =
            require_json_string(property.get("physicalType"), "property.physicalType", None)?;
        let required = property
            .get("required")
            .map(|value| {
                value.as_bool().ok_or_else(|| {
                    AppError::InvalidParameter(format!(
                        "required must be boolean for property {name}"
                    ))
                })
            })
            .transpose()?
            .unwrap_or(false);
        let partition_position = property
            .get("partitionKeyPosition")
            .and_then(Value::as_i64)
            .filter(|position| *position > 0);
        normalized.push(json!({
            "name": name,
            "physicalType": normalize_type(physical_type),
            "required": required,
            "partitionKeyPosition": partition_position
        }));
    }
    Ok(Value::Array(normalized))
}

fn normalize_unity_schema(columns: &[UnityColumn]) -> Result<Value, AppError> {
    let mut ordered = columns.to_vec();
    ordered.sort_by_key(|column| column.position.unwrap_or(i32::MAX));
    let normalized = Value::Array(
        ordered
            .iter()
            .map(|column| {
                let name = column.name.as_deref().unwrap_or_default();
                let physical_type = column
                    .type_text
                    .as_deref()
                    .or(column.type_name.as_deref())
                    .unwrap_or_default();
                json!({
                    "name": name,
                    "physicalType": normalize_type(physical_type),
                    "required": column.nullable == Some(false),
                    "partitionKeyPosition": column.partition_index.map(|index| i64::from(index) + 1)
                })
            })
            .collect(),
    );
    Ok(normalized)
}

fn compare_schema(document: &Value, observed: &Value) -> Result<Value, AppError> {
    let expected = normalized_contract_schema(document)?;
    let expected_columns = expected.as_array().expect("normalized contract array");
    let observed_columns = observed.as_array().expect("normalized Unity array");
    let mut differences = Vec::new();
    let length = expected_columns.len().max(observed_columns.len());
    for index in 0..length {
        match (expected_columns.get(index), observed_columns.get(index)) {
            (Some(expected), Some(actual)) if expected != actual => differences.push(json!({
                "severity": "ERROR",
                "code": "COLUMN_MISMATCH",
                "path": format!("schema[0].properties[{index}]"),
                "expected": expected,
                "actual": actual
            })),
            (Some(expected), None) => differences.push(json!({
                "severity": "ERROR",
                "code": "COLUMN_MISSING",
                "path": format!("schema[0].properties[{index}]"),
                "expected": expected,
                "actual": null
            })),
            (None, Some(actual)) => differences.push(json!({
                "severity": "ERROR",
                "code": "UNEXPECTED_COLUMN",
                "path": format!("columns[{index}]"),
                "expected": null,
                "actual": actual
            })),
            _ => {}
        }
    }
    Ok(Value::Array(differences))
}

fn compare_contract_properties(
    contract: &DataContract,
    table: &UnityTable,
    differences: &mut Value,
) {
    let expected = [
        (CONTRACT_ID_PROPERTY, contract.id.as_str()),
        (CONTRACT_VERSION_PROPERTY, contract.version.as_str()),
        (DOCUMENT_HASH_PROPERTY, contract.document_hash.as_str()),
        (SCHEMA_HASH_PROPERTY, contract.schema_hash.as_str()),
    ];
    let items = differences
        .as_array_mut()
        .expect("differences are always an array");
    for (key, expected_value) in expected {
        if table.properties.get(key).map(String::as_str) != Some(expected_value) {
            items.push(json!({
                "severity": "ERROR",
                "code": "CONTRACT_PROPERTY_MISMATCH",
                "path": format!("properties.{key}"),
                "expected": expected_value,
                "actual": table.properties.get(key)
            }));
        }
    }
}

fn contract_properties(contract: &DataContract) -> BTreeMap<String, String> {
    BTreeMap::from([
        (CONTRACT_ID_PROPERTY.to_owned(), contract.id.clone()),
        (
            CONTRACT_VERSION_PROPERTY.to_owned(),
            contract.version.clone(),
        ),
        (
            DOCUMENT_HASH_PROPERTY.to_owned(),
            contract.document_hash.clone(),
        ),
        (
            SCHEMA_HASH_PROPERTY.to_owned(),
            contract.schema_hash.clone(),
        ),
    ])
}

fn is_error_difference(value: &Value) -> bool {
    value.get("severity").and_then(Value::as_str) == Some("ERROR")
}

async fn load_active_contract(
    client: &deadpool_postgres::Object,
    table: &TableName,
) -> Result<DataContract, AppError> {
    let row = queries::get_active_data_contract()
        .bind(client, &table.catalog, &table.schema, &table.table)
        .opt()
        .await?
        .ok_or_else(|| AppError::NotFound(format!("data contract for {}", table.full_name())))?;
    Ok(DataContract {
        id: row.id,
        catalog_name: row.catalog_name,
        schema_name: row.schema_name,
        table_name: row.table_name,
        version: row.version,
        document: row.document,
        document_hash: row.document_hash,
        schema_hash: row.schema_hash,
        created_by: row.created_by,
        created_at: row.created_at,
        sync_state: row.sync_state,
        unity_table_id: non_empty(row.unity_table_id),
        observed_schema_hash: non_empty(row.observed_schema_hash),
        differences: row.differences,
        checked_at: non_empty(row.checked_at),
    })
}

fn require_json_string<'a>(
    value: Option<&'a Value>,
    field: &str,
    expected: Option<&str>,
) -> Result<&'a str, AppError> {
    let value = value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::InvalidParameter(format!("{field} must be a non-empty string")))?;
    if let Some(expected) = expected
        && value != expected
    {
        return Err(AppError::InvalidParameter(format!(
            "{field} must be {expected}"
        )));
    }
    Ok(value)
}

fn is_semver(version: &str) -> bool {
    parse_version(version).is_some()
}

fn version_is_newer(candidate: &str, current: &str) -> bool {
    matches!(
        (parse_version(candidate), parse_version(current)),
        (Some(candidate), Some(current)) if candidate > current
    )
}

fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let parsed = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(parsed)
}

fn normalize_type(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect()
}

fn require_text(field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::InvalidParameter(format!(
            "{field} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn map_write_error(error: tokio_postgres::Error) -> AppError {
    match error.code() {
        Some(code) if code == &SqlState::UNIQUE_VIOLATION => {
            AppError::Conflict("data contract already exists".to_owned())
        }
        Some(code) if code == &SqlState::CHECK_VIOLATION => {
            AppError::InvalidParameter("data contract violates a constraint".to_owned())
        }
        _ => AppError::Postgres(error),
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document() -> Value {
        json!({
            "apiVersion": "v3.1.0",
            "kind": "DataContract",
            "id": "8a4af65c-422d-5946-97fd-bf22eb79a5c5",
            "name": "vietjet.silver.passengers",
            "version": "1.0.0",
            "status": "active",
            "schema": [{
                "name": "passengers",
                "logicalType": "object",
                "physicalType": "table",
                "physicalName": "vietjet.silver.passengers",
                "properties": [{
                    "name": "passenger_id",
                    "logicalType": "string",
                    "physicalType": "STRING",
                    "required": false
                }]
            }]
        })
    }

    #[test]
    fn validates_supported_odcs_table_contract() {
        let table = TableName::parse("vietjet.silver.passengers").unwrap();
        let validated = validate_document(&document(), &table, "1.0.0").unwrap();
        assert_eq!(validated.id, "8a4af65c-422d-5946-97fd-bf22eb79a5c5");
        assert_eq!(validated.normalized_schema.as_array().unwrap().len(), 1);
    }

    #[test]
    fn rejects_contract_for_another_table() {
        let table = TableName::parse("vietjet.silver.bookings").unwrap();
        assert!(validate_document(&document(), &table, "1.0.0").is_err());
    }

    #[test]
    fn reports_physical_schema_drift() {
        let observed = json!([{
            "name": "passenger_id",
            "physicalType": "BIGINT",
            "required": false,
            "partitionKeyPosition": null
        }]);
        let differences = compare_schema(&document(), &observed).unwrap();
        assert_eq!(differences.as_array().unwrap().len(), 1);
        assert_eq!(differences[0]["code"], "COLUMN_MISMATCH");
    }

    #[test]
    fn versions_only_move_forward() {
        assert!(version_is_newer("1.1.0", "1.0.9"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("0.9.0", "1.0.0"));
    }
}
