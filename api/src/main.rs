mod handlers;
mod routes;

use flowforge_common::config::AppConfig;
use flowforge_common::db;
use flowforge_common::queue::TaskQueue;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .json()
        .init();

    info!("FlowForge API starting...");
    let config = AppConfig::from_env();

    // Setup metrics
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let handle = builder.install_recorder()?;

    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    let queue = TaskQueue::new(&config.redis_url).await?;

    let app = routes::create_router(pool, queue, handle);

    let addr = format!("{}:{}", config.api_host, config.api_port);
    info!(addr = %addr, "API server listening");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
