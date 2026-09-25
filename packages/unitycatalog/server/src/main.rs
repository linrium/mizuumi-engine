mod app_state;
mod config;
mod error;
mod features;
mod infrastructure;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    app_state::AppState,
    config::Settings,
    features::{
        health::{DefaultHealthService, health_router},
        hello::{DefaultHelloService, hello_router},
        vending::{DefaultVendingService, vending_router},
    },
    infrastructure::postgres::create_pool,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let settings = Settings::load().context("failed to load configuration")?;
    let pool = create_pool(&settings.postgres).context("failed to create postgres pool")?;

    let state = AppState {
        health: Arc::new(DefaultHealthService::new(pool.clone())),
        hello: Arc::new(DefaultHelloService::new(pool)),
        vending: Arc::new(
            DefaultVendingService::new(settings.vending.clone())
                .context("failed to create vending service")?,
        ),
    };

    let app = Router::new()
        .merge(health_router())
        .merge(hello_router())
        .merge(vending_router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::new(settings.server.host, settings.server.port);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
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
}
