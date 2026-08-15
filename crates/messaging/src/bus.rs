use async_trait::async_trait;
use flowforge_common::{Result, TaskCompletionMessage, TaskDispatchMessage};
use uuid::Uuid;

pub struct SubjectBuilder;

impl SubjectBuilder {
    pub fn task_dispatch(org_id: Uuid, proj_id: Uuid) -> String {
        format!("flowforge.{}.{}.task.dispatch", org_id, proj_id)
    }

    pub fn task_events(org_id: Uuid, proj_id: Uuid) -> String {
        format!("flowforge.{}.{}.task.events", org_id, proj_id)
    }

    pub fn workflow_events(org_id: Uuid, proj_id: Uuid) -> String {
        format!("flowforge.{}.{}.workflow.events", org_id, proj_id)
    }

    pub fn all_task_dispatches() -> String {
        "flowforge.*.*.task.dispatch".to_string()
    }
}

#[async_trait]
pub trait MessageBus: Send + Sync {
    async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()>;
    async fn publish_task_dispatch(&self, msg: &TaskDispatchMessage) -> Result<()>;
    async fn publish_task_completion(&self, msg: &TaskCompletionMessage) -> Result<()>;
    async fn pull_next_task(&self, timeout_ms: u64) -> Result<Option<TaskDispatchMessage>>;
}
