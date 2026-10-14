use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DuckMetricsAggregation {
    P50,
    P90,
    #[default]
    P95,
    P99,
}

impl DuckMetricsAggregation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::P50 => "p50",
            Self::P90 => "p90",
            Self::P95 => "p95",
            Self::P99 => "p99",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetDuckMetricsQuery {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<u32>,
    #[serde(default)]
    pub aggregation: DuckMetricsAggregation,
}

#[derive(Debug, Clone)]
pub struct DuckMetricFilter {
    pub compute_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuckMetricDefinition {
    pub query_name: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
}

#[derive(Debug, Serialize)]
pub struct DuckMetricsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_id: Option<String>,
    pub start: i64,
    pub end: i64,
    pub step: u32,
    pub aggregation: DuckMetricsAggregation,
    pub definitions: Vec<DuckMetricDefinition>,
    pub data: Value,
}
