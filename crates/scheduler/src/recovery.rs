use flowforge_common::{Result, TaskState};
use flowforge_observability::MetricsRegistry;
use flowforge_persistence::Repository;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct StaleLeaseDetector<R: Repository> {
    repo: Arc<R>,
    interval: Duration,
}

impl<R: Repository + 'static> StaleLeaseDetector<R> {
    pub fn new(repo: Arc<R>, interval: Duration) -> Self {
        Self { repo, interval }
    }

    pub async fn run_loop<F: Fn() -> bool + Send + Sync + 'static>(
        &self,
        is_leader: F,
        cancel_token: CancellationToken,
    ) {
        info!("Starting Stale Lease and Dead Worker recovery loop");

        while !cancel_token.is_cancelled() {
            if is_leader() {
                if let Err(e) = self.sweep_stale_leases().await {
                    error!(error = %e, "Error during stale lease sweep");
                }
            }

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.interval) => {}
            }
        }

        info!("Stale lease detector stopped");
    }

    pub async fn sweep_stale_leases(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let stale_leases = self.repo.find_stale_task_leases(now).await?;

        for lease in stale_leases {
            warn!(
                task_run_id = %lease.task_run_id,
                worker_id = %lease.worker_id,
                "Detected expired lease from inactive/lost worker"
            );

            // 1. Fetch task run
            let task_run = self.repo.get_task_run(lease.task_run_id).await?;

            if task_run.status == TaskState::Running || task_run.status == TaskState::Dispatched {
                MetricsRegistry::record_task_lost(&task_run.task_id);

                if task_run.attempt_count < task_run.max_attempts {
                    info!(
                        task_id = %task_run.task_id,
                        attempt = task_run.attempt_count,
                        max = task_run.max_attempts,
                        "Requeueing lost task for retry"
                    );
                    MetricsRegistry::record_task_retry(&task_run.task_id);

                    self.repo
                        .update_task_run_status(
                            task_run.id,
                            TaskState::Ready,
                            None,
                            None,
                            Some(format!("Worker '{}' lease expired (lost)", lease.worker_id)),
                        )
                        .await?;
                } else {
                    warn!(
                        task_id = %task_run.task_id,
                        "Max retry attempts exhausted for lost task, moving to DEAD_LETTER"
                    );

                    self.repo
                        .update_task_run_status(
                            task_run.id,
                            TaskState::DeadLetter,
                            None,
                            None,
                            Some("Task lost and all retry attempts exhausted".to_string()),
                        )
                        .await?;

                    self.repo
                        .route_to_dlq(
                            task_run.workflow_run_id,
                            task_run.id,
                            &task_run.task_id,
                            "WORKER_LOST_MAX_RETRIES_EXCEEDED",
                            task_run.attempt_count,
                            serde_json::json!({ "worker_id": lease.worker_id }),
                            Some("Lease expired and max attempts reached".to_string()),
                        )
                        .await?;
                }

                // 2. Release stale lease
                self.repo.release_task_lease(lease.task_run_id).await?;
            }
        }

        Ok(())
    }
}
