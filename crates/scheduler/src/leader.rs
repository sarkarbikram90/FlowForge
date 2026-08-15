use flowforge_persistence::Repository;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub struct LeaderElector<R: Repository> {
    repo: Arc<R>,
    service_name: String,
    node_id: String,
    lease_duration_secs: u64,
    renew_interval: Duration,
    is_leader: Arc<AtomicBool>,
}

impl<R: Repository> LeaderElector<R> {
    pub fn new(
        repo: Arc<R>,
        service_name: &str,
        node_id: &str,
        lease_duration_secs: u64,
        renew_interval: Duration,
    ) -> Self {
        Self {
            repo,
            service_name: service_name.to_string(),
            node_id: node_id.to_string(),
            lease_duration_secs,
            renew_interval,
            is_leader: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::SeqCst)
    }

    pub async fn run_election_loop(&self, cancel_token: CancellationToken) {
        info!(node_id = %self.node_id, "Starting Leader Election loop for {}", self.service_name);

        while !cancel_token.is_cancelled() {
            let acquired = match self
                .repo
                .try_acquire_scheduler_leader(
                    &self.service_name,
                    &self.node_id,
                    self.lease_duration_secs,
                )
                .await
            {
                Ok(status) => status,
                Err(e) => {
                    error!(node_id = %self.node_id, "Leader election query failed: {}", e);
                    false
                }
            };

            let previously_leader = self.is_leader.swap(acquired, Ordering::SeqCst);

            if acquired && !previously_leader {
                info!(node_id = %self.node_id, "Elected as LEADER for {}", self.service_name);
            } else if !acquired && previously_leader {
                warn!(node_id = %self.node_id, "Lost leadership lease for {}", self.service_name);
            }

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.renew_interval) => {}
            }
        }

        if self.is_leader.load(Ordering::SeqCst) {
            info!(node_id = %self.node_id, "Stepping down from leadership gracefully");
            let _ = self
                .repo
                .step_down_scheduler_leader(&self.service_name, &self.node_id)
                .await;
            self.is_leader.store(false, Ordering::SeqCst);
        }
    }
}
