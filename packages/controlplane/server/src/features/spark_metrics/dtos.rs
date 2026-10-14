use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SparkMetricFilter {
    pub compute_id: String,
    pub application_name: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SparkMetricKind {
    Histogram,
    Gauge,
    Counter,
}

#[derive(Debug, Clone, Serialize)]
pub struct SparkMetricDefinition {
    pub query_name: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub kind: SparkMetricKind,
    pub comparable: bool,
}

#[derive(Debug, Serialize)]
pub struct SparkMetricsResponse {
    pub compute_id: String,
    pub start: i64,
    pub end: i64,
    pub step: u32,
    pub aggregation: crate::features::duck_metrics::DuckMetricsAggregation,
    pub definitions: Vec<SparkMetricDefinition>,
    pub data: Value,
}
