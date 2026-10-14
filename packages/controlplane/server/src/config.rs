use std::{net::IpAddr, path::PathBuf};

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: ServerSettings,
    #[serde(default)]
    pub postgres: PostgresSettings,
    #[serde(default)]
    pub unity_catalog: UnityCatalogSettings,
    #[serde(default)]
    pub duck_query: DuckQuerySettings,
    #[serde(default)]
    pub auth: AuthSettings,
    #[serde(default)]
    pub chronicle: ChronicleSettings,
    #[serde(default)]
    pub signoz: SignozSettings,
    #[serde(default)]
    pub compute: ComputeSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeSettings {
    #[serde(default = "default_compute_namespace")]
    pub namespace: String,
    #[serde(default = "default_compute_label_selector")]
    pub label_selector: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignozSettings {
    #[serde(default = "default_signoz_endpoint")]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_signoz_timeout_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChronicleSettings {
    #[serde(default = "default_chronicle_enabled")]
    pub enabled: bool,
    #[serde(default = "default_chronicle_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_chronicle_service_name")]
    pub service_name: String,
    #[serde(default = "default_chronicle_timeout_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    #[serde(default = "default_auth_issuer")]
    pub issuer: String,
    #[serde(default = "default_auth_audience")]
    pub audience: String,
    pub jwks_url: Option<String>,
    pub ca_certificate: Option<PathBuf>,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    pub bootstrap_token: Option<String>,
    #[serde(default = "default_bootstrap_enabled")]
    pub bootstrap_enabled: bool,
    #[serde(default = "default_bootstrap_token_file")]
    pub bootstrap_token_file: Option<PathBuf>,
    #[serde(default = "default_reader_role")]
    pub reader_role: String,
    #[serde(default = "default_writer_role")]
    pub writer_role: String,
    #[serde(default = "default_admin_role")]
    pub admin_role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DuckQuerySettings {
    #[serde(default = "default_duck_query_url")]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnityCatalogSettings {
    #[serde(default = "default_unity_catalog_url")]
    pub url: String,
    #[serde(default = "default_unity_catalog_token_url")]
    pub token_url: String,
    #[serde(default)]
    pub ca_certificate: Option<PathBuf>,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    #[serde(default = "default_host")]
    pub host: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresSettings {
    #[serde(default = "default_postgres_host")]
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    #[serde(default = "default_postgres_user")]
    pub user: String,
    #[serde(default = "default_postgres_password")]
    pub password: String,
    #[serde(default = "default_postgres_database")]
    pub database: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path = std::env::var_os("CONTROLPLANE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config.toml"));

        Config::builder()
            .add_source(File::from(config_path).required(false))
            .add_source(Environment::with_prefix("CONTROLPLANE").separator("__"))
            .build()?
            .try_deserialize()
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for PostgresSettings {
    fn default() -> Self {
        Self {
            host: default_postgres_host(),
            port: default_postgres_port(),
            user: default_postgres_user(),
            password: default_postgres_password(),
            database: default_postgres_database(),
            pool_size: default_pool_size(),
        }
    }
}

impl Default for UnityCatalogSettings {
    fn default() -> Self {
        Self {
            url: default_unity_catalog_url(),
            token_url: default_unity_catalog_token_url(),
            ca_certificate: None,
            client_id: String::new(),
            client_secret: String::new(),
        }
    }
}

impl Default for DuckQuerySettings {
    fn default() -> Self {
        Self {
            url: default_duck_query_url(),
        }
    }
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            enabled: default_auth_enabled(),
            issuer: default_auth_issuer(),
            audience: default_auth_audience(),
            jwks_url: None,
            ca_certificate: None,
            clock_skew_seconds: default_clock_skew_seconds(),
            bootstrap_token: None,
            bootstrap_enabled: default_bootstrap_enabled(),
            bootstrap_token_file: default_bootstrap_token_file(),
            reader_role: default_reader_role(),
            writer_role: default_writer_role(),
            admin_role: default_admin_role(),
        }
    }
}

impl Default for ChronicleSettings {
    fn default() -> Self {
        Self {
            enabled: default_chronicle_enabled(),
            endpoint: default_chronicle_endpoint(),
            service_name: default_chronicle_service_name(),
            timeout_ms: default_chronicle_timeout_ms(),
        }
    }
}

impl Default for SignozSettings {
    fn default() -> Self {
        Self {
            endpoint: default_signoz_endpoint(),
            api_key: String::new(),
            timeout_ms: default_signoz_timeout_ms(),
        }
    }
}

impl Default for ComputeSettings {
    fn default() -> Self {
        Self {
            namespace: default_compute_namespace(),
            label_selector: default_compute_label_selector(),
        }
    }
}

fn default_host() -> IpAddr {
    [0, 0, 0, 0].into()
}

fn default_port() -> u16 {
    8080
}

fn default_postgres_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_postgres_port() -> u16 {
    5432
}

fn default_postgres_user() -> String {
    "controlplane".to_owned()
}

fn default_postgres_password() -> String {
    "controlplane".to_owned()
}

fn default_postgres_database() -> String {
    "controlplane".to_owned()
}

fn default_pool_size() -> usize {
    16
}

fn default_unity_catalog_url() -> String {
    "http://127.0.0.1:8081".to_owned()
}

fn default_unity_catalog_token_url() -> String {
    "http://127.0.0.1:8080/realms/sovico/protocol/openid-connect/token".to_owned()
}

fn default_duck_query_url() -> String {
    "http://127.0.0.1:9495/api/query".to_owned()
}

fn default_auth_enabled() -> bool {
    true
}

fn default_auth_issuer() -> String {
    "https://auth.mizuumi.test/realms/sovico".to_owned()
}

fn default_auth_audience() -> String {
    "controlplane".to_owned()
}

fn default_clock_skew_seconds() -> u64 {
    30
}

fn default_bootstrap_enabled() -> bool {
    true
}

fn default_bootstrap_token_file() -> Option<PathBuf> {
    Some(PathBuf::from("bootstrap-token"))
}

fn default_reader_role() -> String {
    "controlplane-reader".to_owned()
}

fn default_writer_role() -> String {
    "controlplane-writer".to_owned()
}

fn default_admin_role() -> String {
    "controlplane-admin".to_owned()
}

fn default_chronicle_enabled() -> bool {
    false
}

fn default_chronicle_endpoint() -> String {
    "http://chronicle.chronicle.svc.cluster.local".to_owned()
}

fn default_chronicle_service_name() -> String {
    "controlplane-server".to_owned()
}

fn default_signoz_endpoint() -> String {
    "http://signoz.signoz.svc.cluster.local:8080".to_owned()
}

fn default_signoz_timeout_ms() -> u64 {
    10_000
}

fn default_compute_namespace() -> String {
    "duck-server".to_owned()
}

fn default_compute_label_selector() -> String {
    "app.kubernetes.io/name=duck-server".to_owned()
}

fn default_chronicle_timeout_ms() -> u64 {
    2000
}
