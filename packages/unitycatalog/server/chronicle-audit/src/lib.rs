use std::{
    collections::BTreeMap,
    sync::OnceLock,
    time::{Duration, SystemTime},
};

use opentelemetry::{
    Context, global,
    trace::{SpanKind, Status, TraceContextExt, Tracer},
};
use opentelemetry_http::HeaderInjector;
use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub use chronicle_audit_macros::audit;

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    pub fields: Vec<(String, String)>,
    #[serde(serialize_with = "serialize_time")]
    pub started_at: SystemTime,
    #[serde(serialize_with = "serialize_time")]
    pub finished_at: SystemTime,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn serialize_time<S: serde::Serializer>(
    value: &SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let offset = OffsetDateTime::from(*value);
    let formatted = offset
        .format(&Rfc3339)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&formatted)
}

#[derive(Debug, Clone)]
pub struct ChronicleConfig {
    pub endpoint: String,
    pub service_name: String,
    pub timeout: Duration,
}

#[derive(Clone)]
pub struct ChronicleClient {
    endpoint: String,
    service_name: String,
    http: reqwest::Client,
}

static CLIENT: OnceLock<Option<ChronicleClient>> = OnceLock::new();

impl ChronicleClient {
    pub fn new(config: &ChronicleConfig) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self {
            endpoint: config.endpoint.trim_end_matches('/').to_owned(),
            service_name: config.service_name.clone(),
            http,
        })
    }
}

pub fn init(client: Option<ChronicleClient>) {
    if CLIENT.set(client).is_err() {
        tracing::warn!("chronicle audit client was already initialized; ignoring second init");
    }
}

pub fn record(event: AuditEvent) {
    let Some(Some(client)) = CLIENT.get() else {
        return;
    };
    let client = client.clone();

    tracing::debug!(
        action = %event.action,
        status = %event.status,
        "chronicle audit event"
    );

    let parent_cx = tracing::Span::current().context();
    tokio::spawn(async move {
        send(client, event, parent_cx).await;
    });
}

async fn send(client: ChronicleClient, event: AuditEvent, parent_cx: Context) {
    let tracer = global::tracer("chronicle-audit");
    let span = tracer
        .span_builder("chronicle.audit.send")
        .with_kind(SpanKind::Client)
        .with_attributes([
            opentelemetry::KeyValue::new("audit.action", event.action.clone()),
            opentelemetry::KeyValue::new("audit.status", event.status.clone()),
            opentelemetry::KeyValue::new(
                "server.address",
                client.endpoint.clone(),
            ),
        ])
        .start_with_context(&tracer, &parent_cx);
    let cx = parent_cx.with_span(span);

    let envelope = {
        let mut map: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
        map.insert(
            "source",
            serde_json::Value::String(client.service_name.clone()),
        );
        map.insert(
            "event",
            serde_json::to_value(&event).unwrap_or(serde_json::Value::Null),
        );
        map
    };

    let url = format!("{}/entries", client.endpoint);
    let mut request = match client.http.post(&url).json(&envelope).build() {
        Ok(r) => r,
        Err(error) => {
            cx.span().set_status(Status::error(error.to_string()));
            tracing::warn!(action = %event.action, %error, "failed to build chronicle request");
            return;
        }
    };

    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(request.headers_mut()));
    });

    let response = client.http.execute(request).await;
    match response {
        Ok(resp) if resp.status().is_success() => {
            cx.span()
                .set_attribute(opentelemetry::KeyValue::new(
                    "http.response.status_code",
                    resp.status().as_u16() as i64,
                ));
            tracing::trace!(action = %event.action, status = %resp.status(), "chronicle entry appended");
        }
        Ok(resp) => {
            cx.span()
                .set_attribute(opentelemetry::KeyValue::new(
                    "http.response.status_code",
                    resp.status().as_u16() as i64,
                ));
            cx.span()
                .set_status(Status::error(format!("http {}", resp.status())));
            tracing::warn!(
                action = %event.action,
                status = %resp.status(),
                "chronicle entry rejected"
            );
        }
        Err(error) => {
            cx.span().set_status(Status::error(error.to_string()));
            tracing::warn!(action = %event.action, %error, "chronicle entry delivery failed");
        }
    }
    cx.span().end();
}

#[doc(hidden)]
pub mod __rt {
    use std::time::SystemTime;

    pub fn now() -> SystemTime {
        SystemTime::now()
    }

    pub fn record(event: super::AuditEvent) {
        super::record(event);
    }

    pub trait AuditOutcome {
        fn outcome(&self) -> (&'static str, Option<String>);
    }

    impl<T, E: std::fmt::Display> AuditOutcome for Result<T, E> {
        fn outcome(&self) -> (&'static str, Option<String>) {
            match self {
                Ok(_) => ("ok", None),
                Err(error) => ("error", Some(format!("{error}"))),
            }
        }
    }

    pub fn outcome<T: AuditOutcome>(value: &T) -> (&'static str, Option<String>) {
        value.outcome()
    }
}
