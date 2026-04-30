mod scheduler;

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

    info!("FlowForge Scheduler starting...");
    let config = AppConfig::from_env();

    // Setup metrics
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let handle = builder.install_recorder()?;

    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    let queue = TaskQueue::new(&config.redis_url).await?;

    // Start metrics server on port 9090
    let metrics_handle = handle.clone();
    tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/metrics",
            axum::routing::get(move || {
                let h = metrics_handle.clone();
                async move { h.render() }
            }),
        );
        let listener = tokio::net::TcpListener::bind("0.0.0.0:9090").await.unwrap();
        info!("Metrics server listening on :9090");
        axum::serve(listener, app).await.unwrap();
    });

    let sched = scheduler::Scheduler::new(pool, queue, config);
    sched.run().await?;

    Ok(())
}
