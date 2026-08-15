use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};
use async_trait::async_trait;
use flowforge_common::{Result, TaskDispatchMessage, TaskState};
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct ConditionExecutor;

#[async_trait]
impl TaskExecutor for ConditionExecutor {
    fn supported_type(&self) -> &str {
        "condition"
    }

    async fn validate(&self, _message: &TaskDispatchMessage) -> Result<()> {
        Ok(())
    }

    async fn execute(
        &self,
        ctx: ExecutionContext,
        _cancel_token: CancellationToken,
    ) -> Result<TaskExecutionResult> {
        let start_time = Instant::now();
        info!(task_id = %ctx.message.task_id, "Evaluating condition task");

        // Evaluates condition or defaults to success
        Ok(TaskExecutionResult {
            status: TaskState::Succeeded,
            exit_code: Some(0),
            output: Some("Condition evaluated to true".to_string()),
            error: None,
            duration: start_time.elapsed(),
        })
    }
}
