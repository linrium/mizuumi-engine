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
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnityCatalogSettings {
    #[serde(default = "default_unity_catalog_url")]
    pub url: String,
    #[serde(default = "default_unity_catalog_token_url")]
    pub token_url: String,
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
            client_id: String::new(),
            client_secret: String::new(),
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
