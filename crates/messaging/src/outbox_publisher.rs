use crate::bus::MessageBus;
use flowforge_persistence::Repository;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub struct OutboxPublisher<R: Repository, B: MessageBus> {
    repo: Arc<R>,
    bus: Arc<B>,
    interval: Duration,
}

impl<R: Repository + 'static, B: MessageBus + 'static> OutboxPublisher<R, B> {
    pub fn new(repo: Arc<R>, bus: Arc<B>, interval: Duration) -> Self {
        Self {
            repo,
            bus,
            interval,
        }
    }

    pub async fn run_loop(&self, cancel_token: CancellationToken) {
        info!("Starting Outbox Publisher loop");
        while !cancel_token.is_cancelled() {
            if let Err(e) = self.process_pending_messages().await {
                error!("Error processing outbox messages: {}", e);
            }
            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.interval) => {}
            }
        }
        info!("Outbox Publisher loop stopped");
    }

    pub async fn process_pending_messages(&self) -> flowforge_common::Result<usize> {
        let pending = self.repo.fetch_pending_outbox(50).await?;
        let count = pending.len();

        for record in pending {
            let payload_bytes = serde_json::to_vec(&record.payload)
                .map_err(|e| flowforge_common::FlowForgeError::Internal(e.to_string()))?;

            // Publish to message bus
            self.bus.publish(&record.topic, &payload_bytes).await?;

            // Mark published in database
            self.repo.mark_outbox_published(record.id).await?;
        }

        Ok(count)
    }
}
