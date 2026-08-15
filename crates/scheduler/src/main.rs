use flowforge_common::PlatformConfig;
use flowforge_messaging::InMemoryMessageBus;
use flowforge_observability::{init_tracing, MetricsRegistry};
use flowforge_persistence::PostgresDatabase;
use flowforge_scheduler::{LeaderElector, SchedulerEngine, StaleLeaseDetector};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("flowforge-scheduler");
    let _ = MetricsRegistry::init();

    let config = PlatformConfig::default();
    info!(
        "Starting FlowForge HA Scheduler with database: {}",
        config.database_url
    );

    let cancel_token = CancellationToken::new();

    // Setup graceful shutdown handler
    let cancel_token_clone = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Shutdown signal received, starting graceful termination...");
        cancel_token_clone.cancel();
    });

    let db = PostgresDatabase::connect(&config.database_url).await?;
    let repo = Arc::new(db);
    let bus = Arc::new(InMemoryMessageBus::new());

    let instance_id = format!("sched-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let leader_elector = Arc::new(LeaderElector::new(
        repo.clone(),
        "flowforge-scheduler",
        &instance_id,
        config.lease_duration_secs,
        Duration::from_secs(2),
    ));

    let leader_token = cancel_token.clone();
    let elector_clone = leader_elector.clone();
    let elector_handle = tokio::spawn(async move {
        elector_clone.run_election_loop(leader_token).await;
    });

    let engine = Arc::new(SchedulerEngine::new(
        repo.clone(),
        bus.clone(),
        Duration::from_secs(config.scheduler_interval_secs),
    ));

    let engine_token = cancel_token.clone();
    let elector_for_engine = leader_elector.clone();
    let engine_handle = tokio::spawn(async move {
        engine
            .run_loop(move || elector_for_engine.is_leader(), engine_token)
            .await;
    });

    let detector = Arc::new(StaleLeaseDetector::new(
        repo.clone(),
        Duration::from_secs(5),
    ));

    let detector_token = cancel_token.clone();
    let elector_for_detector = leader_elector.clone();
    let detector_handle = tokio::spawn(async move {
        detector
            .run_loop(move || elector_for_detector.is_leader(), detector_token)
            .await;
    });

    info!(
        "FlowForge HA Scheduler operational. Leader election, engine, and recovery loops active."
    );

    let _ = tokio::join!(elector_handle, engine_handle, detector_handle);
    info!("FlowForge HA Scheduler shutdown complete.");

    Ok(())
}
