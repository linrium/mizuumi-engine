use deadpool_postgres::{Config, CreatePoolError, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

use crate::config::PostgresSettings;

pub fn create_pool(settings: &PostgresSettings) -> Result<Pool, CreatePoolError> {
    let mut config = Config::new();
    config.host = Some(settings.host.clone());
    config.port = Some(settings.port);
    config.user = Some(settings.user.clone());
    config.password = Some(settings.password.clone());
    config.dbname = Some(settings.database.clone());
    config.pool = Some(deadpool_postgres::PoolConfig {
        max_size: settings.pool_size,
        ..Default::default()
    });
    config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    config.create_pool(Some(Runtime::Tokio1), NoTls)
}
