mod config;
mod error;
mod infra;
mod middleware;
mod modules;
mod routes;
mod state;

use std::net::SocketAddr;

use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::routes::build_router;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "investiq_api=info,tower_http=info,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = Config::from_env()?;
    tracing::info!(
        env = %config.app_env,
        rate_limit_rps = config.rate_limit_rps,
        ai_remote = config.ai_api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false),
        "starting InvestIQ AI API"
    );

    let state = AppState::new(config.clone()).await?;

    sqlx::migrate!("./migrations")
        .run(&state.db)
        .await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;

    // Initial + periodic NSE IPO sync (non-fatal on failure)
    {
        let sync_state = state.clone();
        let interval_secs = config.ipo_sync_interval_secs.max(60);
        tokio::spawn(async move {
            loop {
                let mut redis = sync_state.redis.clone();
                match crate::infra::nse_ipo::sync_ipos(
                    sync_state.db(),
                    &mut redis,
                    sync_state.nse(),
                )
                .await
                {
                    Ok(n) => tracing::info!(synced = n, "ipo sync tick"),
                    Err(e) => tracing::error!(error = %e, "ipo sync tick failed"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
            }
        });
    }

    let cors = build_cors(&config)?;

    let app = build_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

fn build_cors(config: &Config) -> anyhow::Result<CorsLayer> {
    if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
        if config.is_production() {
            anyhow::bail!("CORS_ORIGINS must not be * in production");
        }
        return Ok(CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(tower_http::cors::Any));
    }

    let origins: Result<Vec<HeaderValue>, _> = config
        .cors_origins
        .iter()
        .map(|o| o.parse::<HeaderValue>())
        .collect();
    let origins = origins.map_err(|e| anyhow::anyhow!("invalid CORS origin: {e}"))?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true))
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
    tracing::info!("shutdown signal received");
}
