use flowforge_common::Result;
use flowforge_persistence::Repository;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub struct LeaderElector<R: Repository> {
    repo: Arc<R>,
    service_name: String,
    instance_id: String,
    lease_duration_secs: u64,
    heartbeat_interval: Duration,
    is_leader: Arc<AtomicBool>,
}

impl<R: Repository + 'static> LeaderElector<R> {
    pub fn new(
        repo: Arc<R>,
        service_name: &str,
        instance_id: &str,
        lease_duration_secs: u64,
        heartbeat_interval: Duration,
    ) -> Self {
        Self {
            repo,
            service_name: service_name.to_string(),
            instance_id: instance_id.to_string(),
            lease_duration_secs,
            heartbeat_interval,
            is_leader: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::SeqCst)
    }

    pub async fn run_election_loop(&self, cancel_token: CancellationToken) {
        info!(instance_id = %self.instance_id, "Starting Leader Election loop");

        while !cancel_token.is_cancelled() {
            match self
                .repo
                .try_acquire_scheduler_leader(
                    &self.service_name,
                    &self.instance_id,
                    self.lease_duration_secs,
                )
                .await
            {
                Ok(acquired) => {
                    let was_leader = self.is_leader.swap(acquired, Ordering::SeqCst);
                    if acquired && !was_leader {
                        info!(instance_id = %self.instance_id, "Acquired leadership lease! Stepped up to LEADER.");
                    } else if !acquired && was_leader {
                        warn!(instance_id = %self.instance_id, "Lost leadership lease! Stepped down to FOLLOWER.");
                    }
                }
                Err(e) => {
                    warn!(instance_id = %self.instance_id, error = %e, "Error attempting leader lease acquisition");
                    self.is_leader.store(false, Ordering::SeqCst);
                }
            }

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.heartbeat_interval) => {}
            }
        }

        // Graceful step down
        if self.is_leader.load(Ordering::SeqCst) {
            info!(instance_id = %self.instance_id, "Stepping down leadership on shutdown");
            let _ = self.repo.step_down_scheduler_leader(&self.service_name, &self.instance_id).await;
            self.is_leader.store(false, Ordering::SeqCst);
        }
    }
}
