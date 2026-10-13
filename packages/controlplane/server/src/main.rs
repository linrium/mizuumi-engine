mod app_state;
mod config;
mod error;
mod features;
mod infrastructure;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{Router, body::Body, http::Request};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

use crate::{
    app_state::AppState,
    config::Settings,
    features::{
        access_requests::{DefaultAccessRequestService, access_request_router},
        data_contracts::{DefaultDataContractService, data_contract_router},
        health::{DefaultHealthService, health_router},
        pipelines::{DefaultPipelineService, pipeline_router},
        policy_templates::{DefaultPolicyTemplateService, policy_template_router},
    },
    infrastructure::{
        postgres::{create_pool, run_migrations},
        unity_catalog::HttpUnityCatalog,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let settings = Settings::load().context("failed to load configuration")?;
    tracing::debug!(
        server.host = %settings.server.host,
        server.port = settings.server.port,
        postgres.host = %settings.postgres.host,
        postgres.port = settings.postgres.port,
        postgres.database = %settings.postgres.database,
        "configuration loaded"
    );

    let pool = create_pool(&settings.postgres).context("failed to create postgres pool")?;
    run_migrations(&pool)
        .await
        .context("failed to run postgres migrations")?;

    let kube_client = kube::Client::try_default()
        .await
        .context("failed to initialize kubernetes client")?;
    let unity_catalog = Arc::new(
        HttpUnityCatalog::new(&settings.unity_catalog)
            .context("failed to configure Unity Catalog client")?,
    );

    let state = AppState {
        access_requests: Arc::new(DefaultAccessRequestService::new(
            pool.clone(),
            unity_catalog.clone(),
        )),
        data_contracts: Arc::new(DefaultDataContractService::new(
            pool.clone(),
            unity_catalog,
            settings.duck_query.url,
        )),
        health: Arc::new(DefaultHealthService::new(pool.clone())),
        pipelines: Arc::new(DefaultPipelineService::new(kube_client)),
        policy_templates: Arc::new(DefaultPolicyTemplateService::new(pool)),
    };
    let app = Router::new()
        .merge(health_router())
        .merge(data_contract_router())
        .merge(policy_template_router())
        .merge(access_request_router())
        .merge(pipeline_router())
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        path = request.uri().path(),
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

    let address = SocketAddr::new(settings.server.host, settings.server.port);
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind {address}"))?;

    tracing::info!(%address, "control plane server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;
    tracing::info!("control plane server stopped");

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
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
