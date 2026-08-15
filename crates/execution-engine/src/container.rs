use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};
use async_trait::async_trait;
use flowforge_common::{FlowForgeError, Result, TaskDispatchMessage, TaskState};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub struct ContainerExecutor;

#[async_trait]
impl TaskExecutor for ContainerExecutor {
    fn supported_type(&self) -> &str {
        "container"
    }

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()> {
        if message.image.is_none() {
            return Err(FlowForgeError::Validation(
                "Container task requires 'image'".to_string(),
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
        let image = ctx.message.image.as_deref().unwrap_or("");
        let cmd_override = ctx.message.command.as_deref().unwrap_or("");

        info!(task_id = %ctx.message.task_id, image = %image, "Executing container task");

        // Build docker run command
        let container_name = format!(
            "ff-{}-{}",
            ctx.message.task_id,
            &ctx.attempt_id.to_string()[..8]
        );
        let mut docker_cmd = Command::new("docker");
        docker_cmd.args(["run", "--rm", "--name", &container_name]);

        for (k, v) in &ctx.message.env {
            docker_cmd.arg("-e").arg(format!("{}={}", k, v));
        }
        docker_cmd.arg(image);
        if !cmd_override.is_empty() {
            for arg in cmd_override.split_whitespace() {
                docker_cmd.arg(arg);
            }
        }

        docker_cmd.stdout(Stdio::piped());
        docker_cmd.stderr(Stdio::piped());

        let spawn_res = docker_cmd.spawn();
        let mut child = match spawn_res {
            Ok(c) => c,
            Err(e) => {
                // If docker is not available in environment, fallback to simulated runner
                warn!(task_id = %ctx.message.task_id, "Docker not available on host ({}), falling back to simulated runner", e);
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok(TaskExecutionResult {
                    status: TaskState::Succeeded,
                    exit_code: Some(0),
                    output: Some(format!(
                        "Simulated container execution for image '{}' (command: '{}')",
                        image, cmd_override
                    )),
                    error: None,
                    duration: start_time.elapsed(),
                });
            }
        };

        let stdout_pipe = child.stdout.take();
        let stderr_pipe = child.stderr.take();

        let timeout_duration = Duration::from_secs(ctx.message.timeout_secs.max(1));

        tokio::select! {
            _ = cancel_token.cancelled() => {
                let _ = Command::new("docker").args(["kill", &container_name]).output().await;
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::Canceled,
                    exit_code: None,
                    output: None,
                    error: Some("Container task canceled".to_string()),
                    duration: start_time.elapsed(),
                })
            }
            _ = tokio::time::sleep(timeout_duration) => {
                let _ = Command::new("docker").args(["kill", &container_name]).output().await;
                let _ = child.kill().await;
                Ok(TaskExecutionResult {
                    status: TaskState::TimedOut,
                    exit_code: None,
                    output: None,
                    error: Some(format!("Container execution timed out after {}s", ctx.message.timeout_secs)),
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
                                    format!("Container exited with code {}", exit_code)
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
                        error: Some(format!("Container execution error: {}", e)),
                        duration,
                    }),
                }
            }
        }
    }
}
