use std::{net::IpAddr, path::PathBuf};

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: ServerSettings,
    #[serde(default)]
    pub postgres: PostgresSettings,
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
    #[serde(default = "default_pg_host")]
    pub host: String,
    #[serde(default = "default_pg_port")]
    pub port: u16,
    #[serde(default = "default_pg_user")]
    pub user: String,
    #[serde(default = "default_pg_password")]
    pub password: String,
    #[serde(default = "default_pg_database")]
    pub database: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path = std::env::var_os("UNITYCATALOG_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config.toml"));

        Config::builder()
            .add_source(File::from(config_path).required(false))
            .add_source(Environment::with_prefix("UNITYCATALOG").separator("__"))
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
            host: default_pg_host(),
            port: default_pg_port(),
            user: default_pg_user(),
            password: default_pg_password(),
            database: default_pg_database(),
            pool_size: default_pool_size(),
        }
    }
}

fn default_host() -> IpAddr {
    [0, 0, 0, 0].into()
}

fn default_port() -> u16 {
    8080
}

fn default_pg_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_pg_port() -> u16 {
    5432
}

fn default_pg_user() -> String {
    "postgres".to_owned()
}

fn default_pg_password() -> String {
    "postgres".to_owned()
}

fn default_pg_database() -> String {
    "postgres".to_owned()
}

fn default_pool_size() -> usize {
    16
}
