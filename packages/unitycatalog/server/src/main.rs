mod app_state;
mod config;
mod error;
mod features;
mod infrastructure;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{Router, body::Body, http::Request, middleware};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app_state::AppState,
    config::Settings,
    features::{
        auth::{AuthService, auth_router, authorize},
        catalogs::{DefaultCatalogService, catalog_router},
        credentials::{DefaultCredentialService, credential_router},
        delta_commits::{DefaultDeltaCommitService, delta_commits_router},
        external_locations::{DefaultExternalLocationService, external_location_router},
        grants::{DefaultGrantService, grant_router},
        health::{DefaultHealthService, health_router},
        hello::{DefaultHelloService, hello_router},
        schemas::{DefaultSchemaService, schema_router},
        tables::{DefaultTableService, table_router},
        temporary_credentials::{DefaultTemporaryCredentialsService, temporary_credentials_router},
        vending::{DefaultVendingService, vending_router},
    },
    infrastructure::postgres::{create_pool, run_migrations},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    tracing::trace!("loading server configuration");
    let settings = Settings::load().context("failed to load configuration")?;
    tracing::debug!(
        server.host = %settings.server.host,
        server.port = settings.server.port,
        postgres.host = %settings.postgres.host,
        postgres.port = settings.postgres.port,
        postgres.database = %settings.postgres.database,
        postgres.pool_size = settings.postgres.pool_size,
        vending.endpoint_url = %settings.vending.endpoint_url,
        auth.enabled = settings.auth.enabled,
        auth.issuer = %settings.auth.issuer,
        auth.audience = %settings.auth.audience,
        "configuration loaded"
    );

    let auth = AuthService::new(&settings.auth).context("failed to initialize authentication")?;

    tracing::trace!("creating postgres connection pool");
    let pool = create_pool(&settings.postgres).context("failed to create postgres pool")?;
    run_migrations(&pool)
        .await
        .context("failed to run postgres migrations")?;
    tracing::debug!("postgres migrations complete");

    let state = AppState {
        catalogs: Arc::new(DefaultCatalogService::new(pool.clone())),
        credentials: Arc::new(DefaultCredentialService::new(
            pool.clone(),
            settings.vending.clone(),
        )),
        delta_commits: Arc::new(DefaultDeltaCommitService::new(pool.clone())),
        external_locations: Arc::new(DefaultExternalLocationService::new(pool.clone())),
        grants: Arc::new(DefaultGrantService::new(pool.clone())),
        health: Arc::new(DefaultHealthService::new(pool.clone())),
        hello: Arc::new(DefaultHelloService::new(pool.clone())),
        schemas: Arc::new(DefaultSchemaService::new(pool.clone())),
        tables: Arc::new(DefaultTableService::new(pool.clone())),
        temporary_credentials: Arc::new(DefaultTemporaryCredentialsService::new(
            pool.clone(),
            settings.vending.clone(),
        )),
        vending: Arc::new(
            DefaultVendingService::new(settings.vending.clone())
                .context("failed to create vending service")?,
        ),
    };
    tracing::trace!("application services initialized");

    let api = Router::new()
        .merge(auth_router())
        .merge(hello_router())
        .merge(vending_router())
        .merge(catalog_router())
        .merge(credential_router())
        .merge(delta_commits_router())
        .merge(external_location_router())
        .merge(grant_router())
        .merge(schema_router())
        .merge(table_router())
        .merge(temporary_credentials_router())
        .layer(middleware::from_fn_with_state(auth, authorize))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!(
                        "api_call",
                        method = %request.method(),
                        path = request.uri().path(),
                        version = ?request.version(),
                        principal = tracing::field::Empty,
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

    let app = Router::new()
        .merge(health_router())
        .merge(api)
        .with_state(state);
    tracing::debug!("HTTP routes configured");

    let addr = SocketAddr::new(settings.server.host, settings.server.port);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;
    tracing::info!("server stopped");

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::debug!("shutdown signal received");
}
