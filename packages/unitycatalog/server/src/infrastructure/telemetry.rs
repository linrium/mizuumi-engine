use std::{sync::OnceLock, time::Instant};

use anyhow::Context;
use axum::{
    body::Body,
    extract::MatchedPath,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use opentelemetry::{
    KeyValue, global,
    metrics::Histogram,
    trace::{TracerProvider as _, noop::NoopTracerProvider},
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use opentelemetry_semantic_conventions::{
    attribute::{
        DEPLOYMENT_ENVIRONMENT_NAME, HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE,
        SERVICE_NAMESPACE,
    },
    resource::{SERVICE_NAME, SERVICE_VERSION},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::TelemetrySettings;

const INSTRUMENTATION_SCOPE: &str = "unitycatalog-server";
const HTTP_SERVER_DURATION: &str = "http.server.request.duration";

pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl TelemetryGuard {
    pub fn shutdown(mut self) {
        if let Some(provider) = self.tracer_provider.take() {
            if let Err(error) = provider.shutdown() {
                tracing::warn!(%error, "tracer provider shutdown failed");
            }
        }
        if let Some(provider) = self.meter_provider.take() {
            if let Err(error) = provider.shutdown() {
                tracing::warn!(%error, "meter provider shutdown failed");
            }
        }
        if let Some(provider) = self.logger_provider.take() {
            if let Err(error) = provider.shutdown() {
                tracing::warn!(%error, "logger provider shutdown failed");
            }
        }
    }
}

pub fn init(settings: &TelemetrySettings) -> anyhow::Result<TelemetryGuard> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,server=debug"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    if !settings.enabled {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
        global::set_tracer_provider(NoopTracerProvider::new());
        return Ok(TelemetryGuard {
            tracer_provider: None,
            meter_provider: None,
            logger_provider: None,
        });
    }

    let resource = Resource::builder()
        .with_attributes([
            KeyValue::new(SERVICE_NAME, settings.service_name.clone()),
            KeyValue::new(SERVICE_NAMESPACE, settings.service_namespace.clone()),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(
                DEPLOYMENT_ENVIRONMENT_NAME,
                settings.deployment_environment.clone(),
            ),
        ])
        .build();

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", settings.otlp_endpoint))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .context("failed to build OTLP span exporter")?;
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    let tracer = tracer_provider.tracer(INSTRUMENTATION_SCOPE);
    global::set_tracer_provider(tracer_provider.clone());

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/metrics", settings.otlp_endpoint))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .context("failed to build OTLP metric exporter")?;
    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(resource.clone())
        .build();
    global::set_meter_provider(meter_provider.clone());

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/logs", settings.otlp_endpoint))
        .with_protocol(Protocol::HttpBinary)
        .build()
        .context("failed to build OTLP log exporter")?;
    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();

    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_trace_layer)
        .with(otel_log_layer)
        .init();

    Ok(TelemetryGuard {
        tracer_provider: Some(tracer_provider),
        meter_provider: Some(meter_provider),
        logger_provider: Some(logger_provider),
    })
}

fn duration_histogram() -> &'static Histogram<f64> {
    static DURATION: OnceLock<Histogram<f64>> = OnceLock::new();
    DURATION.get_or_init(|| {
        global::meter(INSTRUMENTATION_SCOPE)
            .f64_histogram(HTTP_SERVER_DURATION)
            .with_unit("s")
            .with_description("Duration of HTTP server requests")
            .build()
    })
}

pub async fn http_metrics(request: Request<Body>, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_owned())
        .unwrap_or_else(|| request.uri().path().to_owned());

    let start = Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16() as i64;

    duration_histogram().record(
        elapsed,
        &[
            KeyValue::new(HTTP_REQUEST_METHOD, method),
            KeyValue::new(HTTP_ROUTE, route),
            KeyValue::new(HTTP_RESPONSE_STATUS_CODE, status),
        ],
    );

    response.into_response()
}
