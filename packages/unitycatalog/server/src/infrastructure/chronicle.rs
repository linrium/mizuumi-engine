use std::time::Duration;

use anyhow::Context;
use chronicle_audit::{ChronicleClient, ChronicleConfig};

use crate::config::{ChronicleSettings, TelemetrySettings};

pub fn init(chronicle: &ChronicleSettings, telemetry: &TelemetrySettings) -> anyhow::Result<()> {
    if !chronicle.enabled {
        chronicle_audit::init(None);
        return Ok(());
    }

    let config = ChronicleConfig {
        endpoint: chronicle.endpoint.clone(),
        service_name: telemetry.service_name.clone(),
        timeout: Duration::from_millis(chronicle.timeout_ms),
    };
    let client =
        ChronicleClient::new(&config).context("failed to build chronicle audit HTTP client")?;
    chronicle_audit::init(Some(client));
    Ok(())
}
