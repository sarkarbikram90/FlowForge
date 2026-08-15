use async_trait::async_trait;
use flowforge_common::{FlowForgeError, Result, TaskDispatchMessage, TaskState};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::info;
use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};

pub struct WaitExecutor;

#[async_trait]
impl TaskExecutor for WaitExecutor {
    fn supported_type(&self) -> &str {
        "wait"
    }

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()> {
        if message.wait_secs.is_none() {
            return Err(FlowForgeError::Validation(
                "Wait task requires 'waitSecs'".to_string(),
            ));
        }
        Ok(())
    }

    async fn execute(
        &self,
        ctx: ExecutionContext,
        cancel_token: CancellationToken,
    ) -> Result<TaskExecutionResult> {
        let start_time = Instant::now();
        let wait_secs = ctx.message.wait_secs.unwrap_or(5);

        info!(task_id = %ctx.message.task_id, wait_secs, "Executing wait task");

        tokio::select! {
            _ = cancel_token.cancelled() => {
                Ok(TaskExecutionResult {
                    status: TaskState::Canceled,
                    exit_code: None,
                    output: None,
                    error: Some("Wait task canceled".to_string()),
                    duration: start_time.elapsed(),
                })
            }
            _ = tokio::time::sleep(Duration::from_secs(wait_secs)) => {
                Ok(TaskExecutionResult {
                    status: TaskState::Succeeded,
                    exit_code: Some(0),
                    output: Some(format!("Waited {} seconds successfully", wait_secs)),
                    error: None,
                    duration: start_time.elapsed(),
                })
            }
        }
    }
}
