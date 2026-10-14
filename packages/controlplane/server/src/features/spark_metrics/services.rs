use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde_json::{Value, json};

use crate::{
    config::SignozSettings,
    error::AppError,
    features::duck_metrics::{DuckMetricsAggregation, GetDuckMetricsQuery},
};

use super::dtos::{
    SparkMetricDefinition, SparkMetricFilter, SparkMetricKind, SparkMetricsResponse,
};

const DEFAULT_WINDOW_MS: i64 = 60 * 60 * 1_000;
const MAX_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
const DEFAULT_STEP_SECONDS: u32 = 60;
const MIN_STEP_SECONDS: u32 = 10;
const MAX_STEP_SECONDS: u32 = 3_600;

const SPARK_METRICS: [SparkMetricDefinition; 12] = [
    gauge("A", "mizuumi.compute.operation.duration", "s", true),
    gauge("B", "mizuumi.compute.operation.output.rows", "{row}", true),
    counter("C", "mizuumi.compute.operation.count", "{operation}", true),
    histogram("D", "flare.task.duration", "ms", false),
    histogram("E", "flare.task.records_throughput", "{records}/s", false),
    counter("F", "flare.task.shuffle.read_bytes", "By", false),
    counter("G", "flare.task.shuffle.write_bytes", "By", false),
    histogram("H", "flare.stage.executor.run_time", "ms", false),
    counter("I", "flare.stage.input.bytes", "By", false),
    counter("J", "flare.stage.output.bytes", "By", false),
    counter("K", "flare.stage.shuffle.read_bytes", "By", false),
    counter("L", "flare.stage.shuffle.write_bytes", "By", false),
];

const fn histogram(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> SparkMetricDefinition {
    SparkMetricDefinition {
        query_name,
        name,
        unit,
        kind: SparkMetricKind::Histogram,
        comparable,
    }
}

const fn counter(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> SparkMetricDefinition {
    SparkMetricDefinition {
        query_name,
        name,
        unit,
        kind: SparkMetricKind::Counter,
        comparable,
    }
}

const fn gauge(
    query_name: &'static str,
    name: &'static str,
    unit: &'static str,
    comparable: bool,
) -> SparkMetricDefinition {
    SparkMetricDefinition {
        query_name,
        name,
        unit,
        kind: SparkMetricKind::Gauge,
        comparable,
    }
}

#[async_trait]
pub trait SparkMetricsService: Send + Sync {
    async fn query(
        &self,
        query: GetDuckMetricsQuery,
        filter: &SparkMetricFilter,
    ) -> Result<SparkMetricsResponse, AppError>;
}

pub struct DefaultSparkMetricsService {
    client: Client,
    query_url: Url,
    api_key: String,
}

impl DefaultSparkMetricsService {
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
                "Spark metrics queries are limited to 7 days".to_owned(),
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
        application_name: &str,
    ) -> Value {
        let driver = format!("{application_name}-driver");
        let executor = format!("{application_name}-executor");
        let filter_expression = format!("service.name IN ['{driver}', '{executor}']");
        let queries = SPARK_METRICS
            .iter()
            .map(|metric| {
                let aggregation_spec = match metric.kind {
                    SparkMetricKind::Histogram => json!({
                        "metricName": format!("{}.bucket", metric.name),
                        "spaceAggregation": aggregation.as_str()
                    }),
                    SparkMetricKind::Gauge => json!({
                        "metricName": format!("{}.max", metric.name),
                        "temporality": "Unspecified",
                        "timeAggregation": "latest",
                        "spaceAggregation": "max"
                    }),
                    SparkMetricKind::Counter => json!({
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
impl SparkMetricsService for DefaultSparkMetricsService {
    #[tracing::instrument(name = "signoz.spark_metrics", skip(self, query, filter))]
    async fn query(
        &self,
        query: GetDuckMetricsQuery,
        filter: &SparkMetricFilter,
    ) -> Result<SparkMetricsResponse, AppError> {
        if self.api_key.is_empty() {
            return Err(AppError::SignozNotConfigured);
        }
        let (start, end, step, aggregation) = Self::resolve_range(&query)?;
        let response = self
            .client
            .post(self.query_url.clone())
            .header("SIGNOZ-API-KEY", &self.api_key)
            .json(&Self::request_body(
                start,
                end,
                step,
                aggregation,
                &filter.application_name,
            ))
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
        Ok(SparkMetricsResponse {
            compute_id: filter.compute_id.clone(),
            start,
            end,
            step,
            aggregation,
            definitions: SPARK_METRICS.to_vec(),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_histogram_and_counter_queries_for_a_spark_application() {
        let body = DefaultSparkMetricsService::request_body(
            1_000,
            2_000,
            60,
            DuckMetricsAggregation::P95,
            "crm-bronze-to-silver",
        );
        let queries = body["compositeQuery"]["queries"].as_array().unwrap();
        assert_eq!(queries.len(), SPARK_METRICS.len());
        assert_eq!(
            queries[0]["spec"]["aggregations"][0]["metricName"],
            "mizuumi.compute.operation.duration.max"
        );
        assert_eq!(
            queries[2]["spec"]["aggregations"][0]["timeAggregation"],
            "increase"
        );
        assert_eq!(
            queries[0]["spec"]["filter"]["expression"],
            "service.name IN ['crm-bronze-to-silver-driver', 'crm-bronze-to-silver-executor']"
        );
    }
}
