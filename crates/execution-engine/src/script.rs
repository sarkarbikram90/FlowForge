use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};
use async_trait::async_trait;
use flowforge_common::{FlowForgeError, Result, TaskDispatchMessage, TaskState};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct ScriptExecutor;

#[async_trait]
impl TaskExecutor for ScriptExecutor {
    fn supported_type(&self) -> &str {
        "script"
    }

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()> {
        if message.script.is_none() && message.command.is_none() {
            return Err(FlowForgeError::Validation(
                "Script task requires 'script' or 'command'".to_string(),
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
        let script_code = ctx
            .message
            .script
            .as_deref()
            .or(ctx.message.command.as_deref())
            .unwrap_or("");

        info!(task_id = %ctx.message.task_id, "Executing script task (python/node fallback)");

        // Try python -c, fallback to shell
        let mut cmd = Command::new("python");
        cmd.args(["-c", script_code]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        for (k, v) in &ctx.message.env {
            cmd.env(k, v);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => {
                #[cfg(target_os = "windows")]
                let mut fallback = {
                    let mut c = Command::new("cmd");
                    c.args(["/C", script_code]);
                    c
                };
                #[cfg(not(target_os = "windows"))]
                let mut fallback = {
                    let mut c = Command::new("sh");
                    c.args(["-c", script_code]);
                    c
                };
                fallback.stdout(Stdio::piped());
                fallback.stderr(Stdio::piped());
                fallback
                    .spawn()
                    .map_err(|e| FlowForgeError::ExecutionFailed {
                        task_id: ctx.message.task_id.clone(),
                        reason: format!("Failed to spawn script execution: {}", e),
                    })?
            }
        };

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let timeout_duration = Duration::from_secs(ctx.message.timeout_secs.max(1));

        tokio::select! {
            _ = cancel_token.cancelled() => {
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::Canceled,
                    exit_code: None,
                    output: None,
                    error: Some("Script task canceled".to_string()),
                    duration: start_time.elapsed(),
                })
            }
            _ = tokio::time::sleep(timeout_duration) => {
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::TimedOut,
                    exit_code: None,
                    output: None,
                    error: Some(format!("Script execution timed out after {}s", ctx.message.timeout_secs)),
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
                                    format!("Script exited with code {}", exit_code)
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
                        error: Some(format!("Script wait error: {}", e)),
                        duration,
                    }),
                }
            }
        }
    }
}
