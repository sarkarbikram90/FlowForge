use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};
use async_trait::async_trait;
use flowforge_common::{FlowForgeError, Result, TaskDispatchMessage, TaskState};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub struct ShellExecutor;

#[async_trait]
impl TaskExecutor for ShellExecutor {
    fn supported_type(&self) -> &str {
        "shell"
    }

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()> {
        if message.command.is_none() && message.script.is_none() {
            return Err(FlowForgeError::Validation(
                "Shell task must provide 'command' or 'script'".to_string(),
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
        let cmd_str = ctx
            .message
            .command
            .as_deref()
            .or(ctx.message.script.as_deref())
            .unwrap_or("");

        info!(task_id = %ctx.message.task_id, cmd = %cmd_str, "Executing shell task");

        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = Command::new("cmd");
            c.args(["/C", cmd_str]);
            c
        };

        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.args(["-c", cmd_str]);
            c
        };

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        for (k, v) in &ctx.message.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| FlowForgeError::ExecutionFailed {
            task_id: ctx.message.task_id.clone(),
            reason: format!("Failed to spawn child process: {}", e),
        })?;

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let timeout_duration = Duration::from_secs(ctx.message.timeout_secs.max(1));

        tokio::select! {
            _ = cancel_token.cancelled() => {
                warn!(task_id = %ctx.message.task_id, "Task execution canceled, killing child process");
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::Canceled,
                    exit_code: None,
                    output: None,
                    error: Some("Execution canceled by user/system request".to_string()),
                    duration: start_time.elapsed(),
                })
            }
            _ = tokio::time::sleep(timeout_duration) => {
                warn!(task_id = %ctx.message.task_id, "Task execution timed out, killing child process");
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::TimedOut,
                    exit_code: None,
                    output: None,
                    error: Some(format!("Execution exceeded timeout of {}s", ctx.message.timeout_secs)),
                    duration: start_time.elapsed(),
                })
            }
            status_res = child.wait() => {
                let duration = start_time.elapsed();
                let mut stdout_buf = Vec::new();
                let mut stderr_buf = Vec::new();

                if let Some(mut out) = stdout_pipe {
                    let _ = out.read_to_end(&mut stdout_buf).await;
                }
                if let Some(mut err) = stderr_pipe {
                    let _ = err.read_to_end(&mut stderr_buf).await;
                }

                let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
                let stderr = String::from_utf8_lossy(&stderr_buf).to_string();

                match status_res {
                    Ok(exit_status) => {
                        let exit_code = exit_status.code().unwrap_or(-1);
                        if exit_status.success() {
                            Ok(TaskExecutionResult {
                                status: TaskState::Succeeded,
                                exit_code: Some(exit_code),
                                output: Some(stdout),
                                error: if stderr.is_empty() { None } else { Some(stderr) },
                                duration,
                            })
                        } else {
                            Ok(TaskExecutionResult {
                                status: TaskState::Failed,
                                exit_code: Some(exit_code),
                                output: Some(stdout),
                                error: Some(if stderr.is_empty() {
                                    format!("Process exited with status code {}", exit_code)
                                } else {
                                    stderr
                                }),
                                duration,
                            })
                        }
                    }
                    Err(e) => Ok(TaskExecutionResult {
                        status: TaskState::Failed,
                        exit_code: None,
                        output: None,
                        error: Some(format!("Process wait error: {}", e)),
                        duration,
                    }),
                }
            }
        }
    }
}
