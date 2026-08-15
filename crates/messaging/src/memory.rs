use crate::bus::MessageBus;
use async_trait::async_trait;
use flowforge_common::{Result, TaskCompletionMessage, TaskDispatchMessage};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct InMemoryMessageBus {
    task_queue: Arc<Mutex<VecDeque<TaskDispatchMessage>>>,
    completion_queue: Arc<Mutex<VecDeque<TaskCompletionMessage>>>,
}

impl InMemoryMessageBus {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MessageBus for InMemoryMessageBus {
    async fn publish(&self, _subject: &str, _payload: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn publish_task_dispatch(&self, msg: &TaskDispatchMessage) -> Result<()> {
        self.task_queue.lock().await.push_back(msg.clone());
        Ok(())
    }

    async fn publish_task_completion(&self, msg: &TaskCompletionMessage) -> Result<()> {
        self.completion_queue.lock().await.push_back(msg.clone());
        Ok(())
    }

    async fn pull_next_task(&self, timeout_ms: u64) -> Result<Option<TaskDispatchMessage>> {
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            {
                let mut queue = self.task_queue.lock().await;
                if let Some(task) = queue.pop_front() {
                    return Ok(Some(task));
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Ok(None);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}
