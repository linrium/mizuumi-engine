use std::time::Duration;

use anyhow::Context;
use chronicle_audit::{ChronicleClient, ChronicleConfig};

use crate::config::ChronicleSettings;

pub fn init(settings: &ChronicleSettings) -> anyhow::Result<()> {
    if !settings.enabled {
        chronicle_audit::init(None);
        return Ok(());
    }

    let config = ChronicleConfig {
        endpoint: settings.endpoint.clone(),
        service_name: settings.service_name.clone(),
        timeout: Duration::from_millis(settings.timeout_ms),
    };
    let client =
        ChronicleClient::new(&config).context("failed to build chronicle audit HTTP client")?;
    chronicle_audit::init(Some(client));
    Ok(())
}
