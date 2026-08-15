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
        "Starting FlowForge Worker connected to: {}",
        config.database_url
    );

    let cancel_token = CancellationToken::new();

    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Shutdown signal received, starting graceful worker draining...");
        cancel_token_clone.cancel();
    });

    let db = PostgresDatabase::connect(&config.database_url).await?;
    let repo = Arc::new(db);
    let bus = Arc::new(InMemoryMessageBus::new());
    let executors = Arc::new(ExecutorRegistry::default());

    let worker_id = std::env::var("WORKER_ID")
        .unwrap_or_else(|_| format!("worker-{}", &uuid::Uuid::new_v4().to_string()[..8]));

    let worker = Arc::new(WorkerAgent::new(
        &worker_id,
        repo,
        bus,
        executors,
        config.worker_concurrency as u32,
    ));

    worker.register().await?;

    let heartbeat_token = cancel_token.clone();
    let worker_for_heartbeat = worker.clone();
    let heartbeat_handle = tokio::spawn(async move {
        worker_for_heartbeat
            .run_heartbeat_loop(heartbeat_token)
            .await;
    });

    let pull_token = cancel_token.clone();
    let worker_for_pull = worker.clone();
    let pull_handle = tokio::spawn(async move {
        worker_for_pull.run_task_pull_loop(pull_token).await;
    });

    info!(
        "FlowForge Worker Agent '{}' is active and consuming tasks",
        worker_id
    );

    let _ = tokio::join!(heartbeat_handle, pull_handle);
    info!(
        "FlowForge Worker Agent '{}' shutdown gracefully.",
        worker_id
    );

    Ok(())
}
