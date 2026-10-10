use std::{
    collections::BTreeMap,
    sync::OnceLock,
    time::{Duration, SystemTime},
};

use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

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

    tokio::spawn(async move {
        send(client, event).await;
    });
}

async fn send(client: ChronicleClient, event: AuditEvent) {
    let mut envelope: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
    envelope.insert(
        "service",
        serde_json::Value::String(client.service_name.clone()),
    );
    envelope.insert(
        "event",
        serde_json::to_value(&event).unwrap_or(serde_json::Value::Null),
    );

    let url = format!("{}/entries", client.endpoint);
    let response = client.http.post(&url).json(&envelope).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            tracing::trace!(action = %event.action, status = %resp.status(), "chronicle entry appended");
        }
        Ok(resp) => {
            tracing::warn!(
                action = %event.action,
                status = %resp.status(),
                "chronicle entry rejected"
            );
        }
        Err(error) => {
            tracing::warn!(action = %event.action, %error, "chronicle entry delivery failed");
        }
    }
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
