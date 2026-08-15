use async_trait::async_trait;
use flowforge_common::{Result, TaskDispatchMessage, TaskState};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub task_run_id: Uuid,
    pub attempt_id: Uuid,
    pub worker_id: String,
    pub message: TaskDispatchMessage,
}

#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    pub status: TaskState,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration: Duration,
}

#[async_trait]
pub trait TaskExecutor: Send + Sync {
    fn supported_type(&self) -> &str;

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()>;

    async fn execute(
        &self,
        ctx: ExecutionContext,
        cancel_token: CancellationToken,
    ) -> Result<TaskExecutionResult>;
}
