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
use tracing_subscriber::EnvFilter;

use crate::{
    app_state::AppState,
    config::Settings,
    features::{
        access_requests::{DefaultAccessRequestService, access_request_router},
        auth::{AuthService, auth_router, authorize},
        computes::{DefaultComputeService, compute_router},
        data_contracts::{DefaultDataContractService, data_contract_router},
        duck_metrics::{DefaultDuckMetricsService, duck_metrics_router},
        health::{DefaultHealthService, health_router},
        pipelines::{DefaultPipelineService, pipeline_router},
        policy_templates::{DefaultPolicyTemplateService, policy_template_router},
        spark_metrics::DefaultSparkMetricsService,
    },
    infrastructure::{
        chronicle,
        postgres::{create_pool, run_migrations},
        unity_catalog::HttpUnityCatalog,
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let settings = Settings::load().context("failed to load configuration")?;
    chronicle::init(&settings.chronicle).context("failed to initialize chronicle audit client")?;
    tracing::debug!(
        server.host = %settings.server.host,
        server.port = settings.server.port,
        postgres.host = %settings.postgres.host,
        postgres.port = settings.postgres.port,
        postgres.database = %settings.postgres.database,
        auth.enabled = settings.auth.enabled,
        auth.issuer = %settings.auth.issuer,
        auth.audience = %settings.auth.audience,
        chronicle.enabled = settings.chronicle.enabled,
        chronicle.endpoint = %settings.chronicle.endpoint,
        "configuration loaded"
    );

    let auth = AuthService::new(&settings.auth).context("failed to initialize authentication")?;

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
        computes: Arc::new(DefaultComputeService::new(
            kube_client.clone(),
            &settings.compute,
        )),
        duck_metrics: Arc::new(
            DefaultDuckMetricsService::new(&settings.signoz)
                .context("failed to configure SigNoz metrics client")?,
        ),
        spark_metrics: Arc::new(
            DefaultSparkMetricsService::new(&settings.signoz)
                .context("failed to configure SigNoz Spark metrics client")?,
        ),
        health: Arc::new(DefaultHealthService::new(pool.clone())),
        pipelines: Arc::new(DefaultPipelineService::new(kube_client)),
        policy_templates: Arc::new(DefaultPolicyTemplateService::new(pool)),
    };
    let api = Router::new()
        .merge(auth_router())
        .merge(data_contract_router())
        .merge(compute_router())
        .merge(duck_metrics_router())
        .merge(policy_template_router())
        .merge(access_request_router())
        .merge(pipeline_router())
        .layer(middleware::from_fn_with_state(auth, authorize));
    let app = Router::new()
        .merge(health_router())
        .merge(api)
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        path = request.uri().path(),
                        principal = tracing::field::Empty,
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
