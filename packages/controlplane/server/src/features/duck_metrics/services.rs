use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde_json::{Value, json};

use crate::{config::SignozSettings, error::AppError};

use super::dtos::{
    DuckMetricDefinition, DuckMetricFilter, DuckMetricKind, DuckMetricsAggregation,
    DuckMetricsResponse, GetDuckMetricsQuery,
};

const DEFAULT_WINDOW_MS: i64 = 60 * 60 * 1_000;
const MAX_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const DEFAULT_STEP_SECONDS: u32 = 60;
const MIN_STEP_SECONDS: u32 = 10;
const MAX_STEP_SECONDS: u32 = 3_600;

const DUCK_METRICS: [DuckMetricDefinition; 21] = [
    gauge("A", "mizuumi.compute.operation.duration", "s", true),
    gauge("B", "mizuumi.compute.operation.cpu.time", "s", true),
    gauge("C", "mizuumi.compute.operation.input.rows", "{row}", true),
    gauge("D", "mizuumi.compute.operation.output.rows", "{row}", true),
    gauge("E", "mizuumi.compute.operation.input.bytes", "By", true),
    gauge("F", "mizuumi.compute.operation.output.bytes", "By", true),
    gauge("G", "mizuumi.compute.operation.memory.peak", "By", true),
    gauge("H", "mizuumi.compute.operation.spill", "By", true),
    counter("I", "mizuumi.compute.operation.count", "{operation}", true),
    histogram("J", "duckdb.query.duration", "s", false),
    histogram("K", "duckdb.query.cpu.time", "s", false),
    histogram("L", "duckdb.query.blocked_thread.time", "s", false),
    histogram("M", "duckdb.query.rows.returned", "{row}", false),
    histogram("N", "duckdb.query.rows.scanned", "{row}", false),
    histogram("O", "duckdb.query.cardinality.cumulative", "{row}", false),
    histogram("P", "duckdb.query.result.size", "By", false),
    histogram("Q", "duckdb.query.memory.peak_buffer", "By", false),
    histogram("R", "duckdb.query.temp_directory.peak_size", "By", false),
    histogram("S", "duckdb.query.memory.allocated", "By", false),
    histogram("T", "duckdb.query.io.read", "By", false),
    histogram("U", "duckdb.query.io.written", "By", false),
];

const fn histogram(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> DuckMetricDefinition {
    DuckMetricDefinition {
        query_name,
        name,
        unit,
        kind: DuckMetricKind::Histogram,
        comparable,
    }
}

const fn counter(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> DuckMetricDefinition {
    DuckMetricDefinition {
        query_name,
        name,
        unit,
        kind: DuckMetricKind::Counter,
        comparable,
    }
}

const fn gauge(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> DuckMetricDefinition {
    DuckMetricDefinition {
        query_name,
        name,
        unit,
        kind: DuckMetricKind::Gauge,
        comparable,
    }
}

#[async_trait]
pub trait DuckMetricsService: Send + Sync {
    async fn query(
        &self,
        query: GetDuckMetricsQuery,
        filter: Option<&DuckMetricFilter>,
    ) -> Result<DuckMetricsResponse, AppError>;
}

pub struct DefaultDuckMetricsService {
    client: Client,
    query_url: Url,
    api_key: String,
}

impl DefaultDuckMetricsService {
    pub fn new(settings: &SignozSettings) -> anyhow::Result<Self> {
        let mut query_url = Url::parse(&settings.endpoint)?;
        if !matches!(query_url.scheme(), "http" | "https") {
            anyhow::bail!("SigNoz endpoint must use http or https");
        }
        query_url.set_path("/api/v5/query_range");
        query_url.set_query(None);
        query_url.set_fragment(None);
        let client = Client::builder()
            .timeout(Duration::from_millis(settings.timeout_ms))
            .build()?;
        Ok(Self {
            client,
            query_url,
            api_key: settings.api_key.trim().to_owned(),
        })
    }

    fn resolve_range(
        query: &GetDuckMetricsQuery,
    ) -> Result<(i64, i64, u32, DuckMetricsAggregation), AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::Signoz("system clock is before the Unix epoch".to_owned()))?
            .as_millis() as i64;
        let end = query.end.unwrap_or(now);
        let start = query.start.unwrap_or(end - DEFAULT_WINDOW_MS);
        let step = query.step.unwrap_or(DEFAULT_STEP_SECONDS);

        if start < 0 || end <= start {
            return Err(AppError::InvalidParameter(
                "start and end must be Unix milliseconds with end after start".to_owned(),
            ));
        }
        if end - start > MAX_WINDOW_MS {
            return Err(AppError::InvalidParameter(
                "DuckDB metrics queries are limited to 7 days".to_owned(),
            ));
        }
        if !(MIN_STEP_SECONDS..=MAX_STEP_SECONDS).contains(&step) {
            return Err(AppError::InvalidParameter(format!(
                "step must be between {MIN_STEP_SECONDS} and {MAX_STEP_SECONDS} seconds"
            )));
        }
        Ok((start, end, step, query.aggregation))
    }

    fn request_body(
        start: i64,
        end: i64,
        step: u32,
        aggregation: DuckMetricsAggregation,
        filter: Option<&DuckMetricFilter>,
    ) -> Value {
        let filter_expression = filter.map_or_else(
            || "service.name = 'duck-server'".to_owned(),
            |filter| {
                format!(
                    "service.name = 'duck-server' AND k8s.pod.uid = '{}'",
                    filter.compute_id
                )
            },
        );
        let queries = DUCK_METRICS
            .iter()
            .map(|metric| {
                let aggregation_spec = match metric.kind {
                    DuckMetricKind::Histogram => json!({
                        "metricName": format!("{}.bucket", metric.name),
                        "spaceAggregation": aggregation.as_str()
                    }),
                    DuckMetricKind::Gauge => json!({
                        "metricName": format!("{}.max", metric.name),
                        "temporality": "Unspecified",
                        "timeAggregation": "latest",
                        "spaceAggregation": "max"
                    }),
                    DuckMetricKind::Counter => json!({
                        "metricName": metric.name,
                        "timeAggregation": "increase",
                        "spaceAggregation": "sum"
                    }),
                };
                json!({
                    "type": "builder_query",
                    "spec": {
                        "name": metric.query_name,
                        "signal": "metrics",
                        "stepInterval": step,
                        "aggregations": [aggregation_spec],
                        "filter": {"expression": filter_expression},
                        "legend": metric.name,
                        "disabled": false
                    }
                })
            })
            .collect::<Vec<_>>();
        json!({
            "start": start,
            "end": end,
            "requestType": "time_series",
            "compositeQuery": {"queries": queries}
        })
    }
}

#[async_trait]
impl DuckMetricsService for DefaultDuckMetricsService {
    #[tracing::instrument(name = "signoz.duck_metrics", skip(self, query))]
    async fn query(
        &self,
        query: GetDuckMetricsQuery,
        filter: Option<&DuckMetricFilter>,
    ) -> Result<DuckMetricsResponse, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::SignozNotConfigured);
        }
        let (start, end, step, aggregation) = Self::resolve_range(&query)?;
        let response = self
            .client
            .post(self.query_url.clone())
            .header("SIGNOZ-API-KEY", &self.api_key)
            .json(&Self::request_body(start, end, step, aggregation, filter))
            .send()
            .await
            .map_err(|error| AppError::Signoz(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(AppError::Signoz(format!("upstream returned HTTP {status}")));
        }
        let data = response
            .json()
            .await
            .map_err(|error| AppError::Signoz(error.to_string()))?;
        Ok(DuckMetricsResponse {
            compute_id: filter.map(|value| value.compute_id.clone()),
            start,
            end,
            step,
            aggregation,
            definitions: DUCK_METRICS.to_vec(),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_fixed_signoz_queries_for_every_duckdb_metric() {
        let body = DefaultDuckMetricsService::request_body(
            1_000,
            2_000,
            60,
            DuckMetricsAggregation::P95,
            None,
        );
        let queries = body["compositeQuery"]["queries"].as_array().unwrap();
        assert_eq!(queries.len(), DUCK_METRICS.len());
        assert_eq!(
            queries[0]["spec"]["aggregations"][0]["metricName"],
            "mizuumi.compute.operation.duration.max"
        );
        assert_eq!(
            queries[0]["spec"]["filter"]["expression"],
            "service.name = 'duck-server'"
        );
        assert_eq!(
            queries[0]["spec"]["aggregations"][0]["timeAggregation"],
            "latest"
        );
        assert_eq!(
            queries[8]["spec"]["aggregations"][0]["timeAggregation"],
            "increase"
        );
    }

    #[test]
    fn scopes_signoz_queries_to_a_compute_uid() {
        let filter = DuckMetricFilter {
            compute_id: "28a6502a-90f4-4a82-bddb-cbb79eb9828b".to_owned(),
        };
        let body = DefaultDuckMetricsService::request_body(
            1_000,
            2_000,
            60,
            DuckMetricsAggregation::P95,
            Some(&filter),
        );
        assert_eq!(
            body["compositeQuery"]["queries"][0]["spec"]["filter"]["expression"],
            "service.name = 'duck-server' AND k8s.pod.uid = '28a6502a-90f4-4a82-bddb-cbb79eb9828b'"
        );
    }

    #[test]
    fn rejects_excessive_ranges_and_invalid_steps() {
        let too_wide = GetDuckMetricsQuery {
            start: Some(1),
            end: Some(1 + MAX_WINDOW_MS + 1),
            step: None,
            aggregation: DuckMetricsAggregation::P95,
        };
        assert!(matches!(
            DefaultDuckMetricsService::resolve_range(&too_wide),
            Err(AppError::InvalidParameter(_))
        ));

        let bad_step = GetDuckMetricsQuery {
            start: Some(1),
            end: Some(2),
            step: Some(1),
            aggregation: DuckMetricsAggregation::P95,
        };
        assert!(matches!(
            DefaultDuckMetricsService::resolve_range(&bad_step),
            Err(AppError::InvalidParameter(_))
        ));
    }
}
