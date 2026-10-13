use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use controlplane_queries::queries::data_contracts as queries;
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::time::Instant;
use tokio_postgres::error::SqlState;

use crate::{
    error::AppError,
    infrastructure::unity_catalog::{UnityCatalog, UnityColumn, UnityTable},
};

use super::dtos::{
    DataContract, DataQualityResult, DataQualityRun, DataQualityRuns, PutDataContractRequest,
    RunDataQualityRequest,
};

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
    async fn list_data_quality_runs(&self, full_name: String) -> Result<DataQualityRuns, AppError>;
    async fn run_data_quality(
        &self,
        full_name: String,
        authorization: String,
        request: RunDataQualityRequest,
    ) -> Result<DataQualityRun, AppError>;
}

pub struct DefaultDataContractService {
    pool: Pool,
    unity_catalog: Arc<dyn UnityCatalog>,
    duck_query_url: String,
    http: reqwest::Client,
}

impl DefaultDataContractService {
    pub fn new(pool: Pool, unity_catalog: Arc<dyn UnityCatalog>, duck_query_url: String) -> Self {
        Self {
            pool,
            unity_catalog,
            duck_query_url,
            http: reqwest::Client::new(),
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

    async fn list_data_quality_runs(&self, full_name: String) -> Result<DataQualityRuns, AppError> {
        let table = TableName::parse(&full_name)?;
        let client = self.pool.get().await?;
        load_quality_runs(&client, &table, 20).await
    }

    async fn run_data_quality(
        &self,
        full_name: String,
        authorization: String,
        request: RunDataQualityRequest,
    ) -> Result<DataQualityRun, AppError> {
        require_text("triggered_by", &request.triggered_by)?;
        let table = TableName::parse(&full_name)?;
        let client = self.pool.get().await?;
        let contract = load_active_contract(&client, &table).await?;
        let rules = executable_quality_rules(&contract.document, &table)?;
        let created = queries::create_data_quality_run()
            .bind(
                &client,
                &contract.id,
                &contract.version,
                &request.triggered_by,
            )
            .one()
            .await?;

        let mut passed_count = 0;
        let mut failed_count = 0;
        let mut error_count = 0;
        for rule in rules {
            let result = self.execute_quality_rule(&authorization, rule).await;
            match result.status.as_str() {
                "PASSED" => passed_count += 1,
                "FAILED" => failed_count += 1,
                _ => error_count += 1,
            }
            let message = result.message.as_deref().unwrap_or("");
            queries::create_data_quality_result()
                .bind(
                    &client,
                    &created.id,
                    &result.rule_id,
                    &result.rule_scope,
                    &result.rule_type,
                    &result.metric,
                    &result.severity,
                    &result.status,
                    &result.metric_value.clone().unwrap_or(Value::Null),
                    &result.expectation,
                    &result.duration_ms,
                    &message,
                )
                .await?;
        }

        let status = if error_count > 0 {
            "ERROR"
        } else if failed_count > 0 {
            "FAILED"
        } else {
            "PASSED"
        };
        queries::complete_data_quality_run()
            .bind(
                &client,
                &status,
                &passed_count,
                &failed_count,
                &error_count,
                &"",
                &created.id,
            )
            .await?;

        load_quality_runs(&client, &table, 20)
            .await?
            .quality_runs
            .into_iter()
            .find(|run| run.id == created.id)
            .ok_or_else(|| AppError::DataQuality("completed run could not be loaded".to_owned()))
    }
}

impl DefaultDataContractService {
    async fn execute_quality_rule(
        &self,
        authorization: &str,
        rule: ExecutableQualityRule,
    ) -> DataQualityResult {
        let started = Instant::now();
        let response = self
            .http
            .post(&self.duck_query_url)
            .header(reqwest::header::AUTHORIZATION, authorization)
            .json(&json!({ "sql": rule.query }))
            .send()
            .await;
        let duration_ms = started.elapsed().as_millis().try_into().unwrap_or(i64::MAX);

        let (status, metric_value, message) = match response {
            Ok(response) => {
                let http_status = response.status();
                match response.json::<DuckQueryResponse>().await {
                    Ok(body) if http_status.is_success() => match body.metric_value() {
                        Some(value) => match quality_rule_passes(&rule.expectation, &value) {
                            Ok(true) => ("PASSED", Some(value), None),
                            Ok(false) => (
                                "FAILED",
                                Some(value),
                                Some("Observed value does not satisfy the contract".to_owned()),
                            ),
                            Err(error) => ("ERROR", Some(value), Some(error)),
                        },
                        None => (
                            "ERROR",
                            None,
                            Some("DuckDB returned no metric value".to_owned()),
                        ),
                    },
                    Ok(body) => (
                        "ERROR",
                        None,
                        Some(
                            body.error
                                .unwrap_or_else(|| format!("DuckDB returned {http_status}")),
                        ),
                    ),
                    Err(error) => (
                        "ERROR",
                        None,
                        Some(format!("invalid DuckDB response: {error}")),
                    ),
                }
            }
            Err(error) => ("ERROR", None, Some(error.to_string())),
        };

        DataQualityResult {
            rule_id: rule.id,
            rule_scope: rule.scope,
            rule_type: rule.rule_type,
            metric: rule.metric,
            severity: rule.severity,
            status: status.to_owned(),
            metric_value,
            expectation: rule.expectation,
            duration_ms,
            message,
        }
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

#[derive(Debug)]
struct ExecutableQualityRule {
    id: String,
    scope: String,
    rule_type: String,
    metric: String,
    severity: String,
    expectation: Value,
    query: String,
}

#[derive(Debug, Deserialize)]
struct DuckQueryResponse {
    rows: Option<Vec<Vec<Value>>>,
    error: Option<String>,
}

impl DuckQueryResponse {
    fn metric_value(self) -> Option<Value> {
        self.rows?.into_iter().next()?.into_iter().next()
    }
}

fn executable_quality_rules(
    document: &Value,
    table: &TableName,
) -> Result<Vec<ExecutableQualityRule>, AppError> {
    let schema = document["schema"][0]
        .as_object()
        .ok_or_else(|| AppError::InvalidParameter("schema[0] must be an object".to_owned()))?;
    let table_sql = [
        quote_identifier(&table.catalog),
        quote_identifier(&table.schema),
        quote_identifier(&table.table),
    ]
    .join(".");
    let mut rules = Vec::new();
    let mut ids = BTreeSet::new();

    append_quality_rules(
        schema.get("quality"),
        "schema[0]",
        None,
        &table_sql,
        &mut ids,
        &mut rules,
    )?;
    if let Some(properties) = schema.get("properties").and_then(Value::as_array) {
        for (index, property) in properties.iter().enumerate() {
            let object = property.as_object().ok_or_else(|| {
                AppError::InvalidParameter(format!(
                    "schema[0].properties[{index}] must be an object"
                ))
            })?;
            let name = require_json_string(object.get("name"), "property.name", None)?;
            append_quality_rules(
                object.get("quality"),
                &format!("schema[0].properties[{index}]"),
                Some(name),
                &table_sql,
                &mut ids,
                &mut rules,
            )?;
        }
    }
    Ok(rules)
}

fn append_quality_rules(
    quality: Option<&Value>,
    scope: &str,
    property: Option<&str>,
    table_sql: &str,
    ids: &mut BTreeSet<String>,
    rules: &mut Vec<ExecutableQualityRule>,
) -> Result<(), AppError> {
    let Some(quality) = quality else {
        return Ok(());
    };
    let quality = quality
        .as_array()
        .ok_or_else(|| AppError::InvalidParameter(format!("{scope}.quality must be an array")))?;
    for (index, value) in quality.iter().enumerate() {
        let rule = value.as_object().ok_or_else(|| {
            AppError::InvalidParameter(format!("{scope}.quality[{index}] must be an object"))
        })?;
        let id = require_json_string(rule.get("id"), "quality.id", None)?.to_owned();
        if !ids.insert(id.clone()) {
            return Err(AppError::InvalidParameter(format!(
                "duplicate quality rule id: {id}"
            )));
        }
        let rule_type = rule
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("library");
        if matches!(rule_type, "text" | "custom") {
            continue;
        }
        let expectation = quality_expectation(rule)?;
        let (metric, query) = match rule_type {
            "library" => {
                let metric = require_json_string(rule.get("metric"), "quality.metric", None)?;
                (
                    metric.to_owned(),
                    library_quality_sql(metric, rule, property, table_sql)?,
                )
            }
            "sql" => {
                let query = require_json_string(rule.get("query"), "quality.query", None)?;
                let property_sql = property.map(quote_identifier).unwrap_or_default();
                (
                    "sql".to_owned(),
                    query
                        .replace("{object}", table_sql)
                        .replace("{table}", table_sql)
                        .replace("{property}", &property_sql)
                        .replace("{column}", &property_sql),
                )
            }
            other => {
                return Err(AppError::InvalidParameter(format!(
                    "unsupported quality rule type: {other}"
                )));
            }
        };
        rules.push(ExecutableQualityRule {
            id,
            scope: property
                .map(|name| format!("{scope}.{name}"))
                .unwrap_or_else(|| scope.to_owned()),
            rule_type: rule_type.to_owned(),
            metric,
            severity: rule
                .get("severity")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_owned(),
            expectation,
            query,
        });
    }
    Ok(())
}

fn library_quality_sql(
    metric: &str,
    rule: &Map<String, Value>,
    property: Option<&str>,
    table_sql: &str,
) -> Result<String, AppError> {
    let column = property.map(quote_identifier);
    let query = match (metric, column.as_deref()) {
        ("rowCount", None) => format!("SELECT count(*) AS metric_value FROM {table_sql}"),
        ("nullValues", Some(column)) => format!(
            "SELECT count(*) FILTER (WHERE {column} IS NULL) AS metric_value FROM {table_sql}"
        ),
        ("duplicateValues", Some(column)) => {
            format!("SELECT count(*) - count(DISTINCT {column}) AS metric_value FROM {table_sql}")
        }
        ("duplicateValues", None) => {
            let properties = rule
                .get("arguments")
                .and_then(Value::as_object)
                .and_then(|arguments| arguments.get("properties"))
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    AppError::InvalidParameter(
                        "schema duplicateValues requires arguments.properties".to_owned(),
                    )
                })?;
            let columns = properties
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .filter(|value| !value.trim().is_empty())
                        .map(quote_identifier)
                        .ok_or_else(|| {
                            AppError::InvalidParameter(
                                "duplicateValues properties must be strings".to_owned(),
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            format!(
                "SELECT (SELECT count(*) FROM {table_sql}) - count(*) AS metric_value FROM (SELECT DISTINCT {columns} FROM {table_sql}) distinct_rows"
            )
        }
        ("missingValues", Some(column)) => {
            let values = rule_argument_values(rule, "missingValues")?;
            let predicates = values
                .iter()
                .map(|value| match value {
                    Value::Null => format!("{column} IS NULL"),
                    _ => format!("CAST({column} AS VARCHAR) = {}", sql_literal(value)),
                })
                .collect::<Vec<_>>()
                .join(" OR ");
            format!("SELECT count(*) FILTER (WHERE {predicates}) AS metric_value FROM {table_sql}")
        }
        ("invalidValues", Some(column)) => {
            let arguments = rule
                .get("arguments")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    AppError::InvalidParameter(
                        "invalidValues requires an arguments object".to_owned(),
                    )
                })?;
            let predicate =
                if let Some(values) = arguments.get("validValues").and_then(Value::as_array) {
                    let values = values
                        .iter()
                        .map(sql_literal)
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("CAST({column} AS VARCHAR) NOT IN ({values})")
                } else if let Some(pattern) = arguments.get("pattern").and_then(Value::as_str) {
                    format!(
                        "NOT regexp_matches(CAST({column} AS VARCHAR), {})",
                        sql_literal(&Value::String(pattern.to_owned()))
                    )
                } else {
                    return Err(AppError::InvalidParameter(
                        "invalidValues requires validValues or pattern".to_owned(),
                    ));
                };
            format!(
                "SELECT count(*) FILTER (WHERE {column} IS NOT NULL AND {predicate}) AS metric_value FROM {table_sql}"
            )
        }
        _ => {
            return Err(AppError::InvalidParameter(format!(
                "unsupported {metric} quality metric at {} level",
                if property.is_some() {
                    "property"
                } else {
                    "schema"
                }
            )));
        }
    };
    Ok(query)
}

fn rule_argument_values<'a>(
    rule: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Vec<Value>, AppError> {
    rule.get("arguments")
        .and_then(Value::as_object)
        .and_then(|arguments| arguments.get(name))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::InvalidParameter(format!("quality rule requires arguments.{name}"))
        })
}

const EXPECTATION_KEYS: [&str; 8] = [
    "mustBe",
    "mustNotBe",
    "mustBeGreaterThan",
    "mustBeGreaterOrEqualTo",
    "mustBeLessThan",
    "mustBeLessOrEqualTo",
    "mustBeBetween",
    "mustNotBeBetween",
];

fn quality_expectation(rule: &Map<String, Value>) -> Result<Value, AppError> {
    let values = EXPECTATION_KEYS
        .iter()
        .filter_map(|key| {
            rule.get(*key)
                .map(|value| ((*key).to_owned(), value.clone()))
        })
        .collect::<Map<_, _>>();
    if values.len() != 1 {
        return Err(AppError::InvalidParameter(
            "quality rule must define exactly one expectation operator".to_owned(),
        ));
    }
    Ok(Value::Object(values))
}

fn quality_rule_passes(expectation: &Value, actual: &Value) -> Result<bool, String> {
    let actual = json_number(actual).ok_or_else(|| "metric value must be numeric".to_owned())?;
    let (operator, expected) = expectation
        .as_object()
        .and_then(|values| values.iter().next())
        .ok_or_else(|| "quality expectation is missing".to_owned())?;
    let number = || json_number(expected).ok_or_else(|| "expectation must be numeric".to_owned());
    match operator.as_str() {
        "mustBe" => Ok(actual == number()?),
        "mustNotBe" => Ok(actual != number()?),
        "mustBeGreaterThan" => Ok(actual > number()?),
        "mustBeGreaterOrEqualTo" => Ok(actual >= number()?),
        "mustBeLessThan" => Ok(actual < number()?),
        "mustBeLessOrEqualTo" => Ok(actual <= number()?),
        "mustBeBetween" | "mustNotBeBetween" => {
            let bounds = expected
                .as_array()
                .filter(|bounds| bounds.len() == 2)
                .ok_or_else(|| "between expectation requires two values".to_owned())?;
            let lower =
                json_number(&bounds[0]).ok_or_else(|| "lower bound must be numeric".to_owned())?;
            let upper =
                json_number(&bounds[1]).ok_or_else(|| "upper bound must be numeric".to_owned())?;
            let between = actual >= lower && actual <= upper;
            Ok(if operator == "mustBeBetween" {
                between
            } else {
                !between
            })
        }
        _ => Err(format!("unsupported expectation operator: {operator}")),
    }
}

fn json_number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &Value) -> String {
    let value = match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    };
    format!("'{}'", value.replace('\'', "''"))
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
    validate_contract_metadata(object)?;
    executable_quality_rules(document, table)?;

    Ok(ValidatedDocument {
        id,
        normalized_schema,
    })
}

fn validate_contract_metadata(object: &Map<String, Value>) -> Result<(), AppError> {
    if let Some(team) = object.get("team") {
        let team = team.as_object().ok_or_else(|| {
            AppError::InvalidParameter("team must use the ODCS v3.1 object format".to_owned())
        })?;
        if let Some(members) = team.get("members") {
            for (index, member) in members
                .as_array()
                .ok_or_else(|| {
                    AppError::InvalidParameter("team.members must be an array".to_owned())
                })?
                .iter()
                .enumerate()
            {
                let member = member.as_object().ok_or_else(|| {
                    AppError::InvalidParameter(format!("team.members[{index}] must be an object"))
                })?;
                require_json_string(
                    member.get("username"),
                    &format!("team.members[{index}].username"),
                    None,
                )?;
            }
        }
    }
    validate_named_array(object.get("slaProperties"), "slaProperties", "property")?;
    validate_custom_properties(object.get("customProperties"), "customProperties")?;
    if let Some(schema) = object.get("schema").and_then(Value::as_array) {
        for (index, item) in schema.iter().enumerate() {
            let item = item.as_object().expect("schema item was validated");
            validate_custom_properties(
                item.get("customProperties"),
                &format!("schema[{index}].customProperties"),
            )?;
            if let Some(properties) = item.get("properties").and_then(Value::as_array) {
                for (property_index, property) in properties.iter().enumerate() {
                    let property = property.as_object().expect("property was validated");
                    validate_custom_properties(
                        property.get("customProperties"),
                        &format!("schema[{index}].properties[{property_index}].customProperties"),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_named_array(
    value: Option<&Value>,
    field: &str,
    required_key: &str,
) -> Result<(), AppError> {
    let Some(value) = value else {
        return Ok(());
    };
    for (index, item) in value
        .as_array()
        .ok_or_else(|| AppError::InvalidParameter(format!("{field} must be an array")))?
        .iter()
        .enumerate()
    {
        let item = item.as_object().ok_or_else(|| {
            AppError::InvalidParameter(format!("{field}[{index}] must be an object"))
        })?;
        require_json_string(
            item.get(required_key),
            &format!("{field}[{index}].{required_key}"),
            None,
        )?;
    }
    Ok(())
}

fn validate_custom_properties(value: Option<&Value>, field: &str) -> Result<(), AppError> {
    validate_named_array(value, field, "property")
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

async fn load_quality_runs(
    client: &deadpool_postgres::Object,
    table: &TableName,
    limit: i64,
) -> Result<DataQualityRuns, AppError> {
    let rows = queries::list_data_quality_runs()
        .bind(client, &table.catalog, &table.schema, &table.table, &limit)
        .all()
        .await?;
    let mut quality_runs = Vec::with_capacity(rows.len());
    for row in rows {
        let results = queries::list_data_quality_results()
            .bind(client, &row.id)
            .all()
            .await?
            .into_iter()
            .map(|result| DataQualityResult {
                rule_id: result.rule_id,
                rule_scope: result.rule_scope,
                rule_type: result.rule_type,
                metric: result.metric,
                severity: result.severity,
                status: result.status,
                metric_value: (!result.metric_value.is_null()).then_some(result.metric_value),
                expectation: result.expectation,
                duration_ms: result.duration_ms,
                message: non_empty(result.message),
            })
            .collect();
        quality_runs.push(DataQualityRun {
            id: row.id,
            contract_id: row.contract_id,
            contract_version: row.contract_version,
            engine: row.engine,
            status: row.status,
            trigger_type: row.trigger_type,
            triggered_by: row.triggered_by,
            started_at: row.started_at,
            completed_at: non_empty(row.completed_at),
            passed_count: row.passed_count,
            failed_count: row.failed_count,
            error_count: row.error_count,
            error: non_empty(row.error),
            results,
        });
    }
    Ok(DataQualityRuns { quality_runs })
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
            "team": {
                "name": "VietJet Data",
                "members": [{"username": "steward@vietjet.test", "role": "Data Steward"}]
            },
            "slaProperties": [{"id": "frequency", "property": "frequency", "value": 10}],
            "customProperties": [{"id": "jurisdiction", "property": "jurisdiction", "value": "VN"}],
            "schema": [{
                "name": "passengers",
                "logicalType": "object",
                "physicalType": "table",
                "physicalName": "vietjet.silver.passengers",
                "quality": [{
                    "id": "passengers_not_empty",
                    "metric": "rowCount",
                    "mustBeGreaterThan": 0
                }],
                "properties": [{
                    "name": "passenger_id",
                    "logicalType": "string",
                    "physicalType": "STRING",
                    "required": false,
                    "quality": [{
                        "id": "passenger_id_not_null",
                        "metric": "nullValues",
                        "mustBe": 0
                    }]
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

    #[test]
    fn compiles_odcs_library_quality_rules_to_quoted_sql() {
        let table = TableName::parse("vietjet.silver.passengers").unwrap();
        let rules = executable_quality_rules(&document(), &table).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "passengers_not_empty");
        assert!(
            rules[0]
                .query
                .contains("\"vietjet\".\"silver\".\"passengers\"")
        );
        assert_eq!(rules[1].metric, "nullValues");
        assert!(rules[1].query.contains("\"passenger_id\" IS NULL"));
    }

    #[test]
    fn evaluates_quality_expectations() {
        assert!(quality_rule_passes(&json!({"mustBe": 0}), &json!(0)).unwrap());
        assert!(quality_rule_passes(&json!({"mustBeBetween": [1, 10]}), &json!(5)).unwrap());
        assert!(!quality_rule_passes(&json!({"mustBeLessThan": 3}), &json!(4)).unwrap());
    }
}
