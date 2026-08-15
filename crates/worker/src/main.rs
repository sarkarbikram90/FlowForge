use flowforge_common::PlatformConfig;
use flowforge_execution_engine::ExecutorRegistry;
use flowforge_messaging::InMemoryMessageBus;
use flowforge_observability::{init_tracing, MetricsRegistry};
use flowforge_persistence::PostgresDatabase;
use flowforge_worker::WorkerAgent;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("flowforge-worker");
    let _ = MetricsRegistry::init();

    let config = PlatformConfig::default();
    info!(
        worker_id = %config.worker_id,
        concurrency = config.worker_concurrency,
        "Starting FlowForge Worker Agent"
    );

    let cancel_token = CancellationToken::new();

    // Setup graceful shutdown handler
    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Shutdown signal received, starting graceful worker draining...");
        cancel_token_clone.cancel();
    });

    let db = PostgresDatabase::connect(&config.database_url).await?;
    let repo = Arc::new(db);
    let bus = Arc::new(InMemoryMessageBus::new());
    let executors = Arc::new(ExecutorRegistry::default());

    let worker = Arc::new(WorkerAgent::new(
        &config.worker_id,
        repo.clone(),
        bus.clone(),
        executors.clone(),
        config.worker_concurrency,
    ));

    worker.register().await?;

    let heartbeat_token = cancel_token.clone();
    let worker_for_heartbeat = worker.clone();
    let heartbeat_handle = tokio::spawn(async move {
        worker_for_heartbeat.run_heartbeat_loop(heartbeat_token).await;
    });

    let pull_token = cancel_token.clone();
    let worker_for_pull = worker.clone();
    let pull_handle = tokio::spawn(async move {
        worker_for_pull.run_task_pull_loop(pull_token).await;
    });

    info!("FlowForge Worker Agent active and pulling tasks.");

    let _ = tokio::join!(heartbeat_handle, pull_handle);
    info!("FlowForge Worker Agent shutdown complete.");

    Ok(())
}
