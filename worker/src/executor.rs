use flowforge_common::models::TaskMessage;
use tokio::process::Command;
use tracing::{debug, info};

/// Execute a task by running its shell command.
/// Returns stdout on success, or error message on failure.
pub async fn execute_task(msg: &TaskMessage) -> Result<String, String> {
    info!(
        task_id = %msg.task_id,
        command = %msg.command,
        timeout_secs = msg.timeout_secs,
        "Starting command execution"
    );

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &msg.command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &msg.command]);
        c
    };

    // Set environment variables
    for (key, value) in &msg.env {
        cmd.env(key, value);
    }

    // Execute with timeout
    let timeout = tokio::time::Duration::from_secs(msg.timeout_secs);
    let result = tokio::time::timeout(timeout, cmd.output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            debug!(
                task_id = %msg.task_id,
                exit_code = output.status.code().unwrap_or(-1),
                stdout_len = stdout.len(),
                stderr_len = stderr.len(),
                "Command completed"
            );

            if output.status.success() {
                let combined = if stderr.is_empty() {
                    stdout
                } else {
                    format!("{stdout}\n--- stderr ---\n{stderr}")
                };
                // Truncate output to 64KB to avoid DB bloat
                Ok(combined.chars().take(65536).collect())
            } else {
                let exit_code = output.status.code().unwrap_or(-1);
                Err(format!(
                    "Command exited with code {exit_code}\nstdout: {}\nstderr: {}",
                    stdout.chars().take(8192).collect::<String>(),
                    stderr.chars().take(8192).collect::<String>()
                ))
            }
        }
        Ok(Err(e)) => Err(format!("Failed to execute command: {e}")),
        Err(_) => Err(format!(
            "Task timed out after {} seconds",
            msg.timeout_secs
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_msg(command: &str) -> TaskMessage {
        TaskMessage {
            task_instance_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            dag_id: "test".to_string(),
            task_id: "test-task".to_string(),
            command: command.to_string(),
            attempt: 1,
            max_retries: 3,
            timeout_secs: 10,
            env: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_successful_command() {
        let msg = make_msg("echo hello");
        let result = execute_task(&msg).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("hello"));
    }

    #[tokio::test]
    async fn test_failed_command() {
        let cmd = if cfg!(target_os = "windows") {
            "cmd /C exit 1"
        } else {
            "exit 1"
        };
        let msg = make_msg(cmd);
        let result = execute_task(&msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout() {
        let cmd = if cfg!(target_os = "windows") {
            "ping -n 30 127.0.0.1"
        } else {
            "sleep 30"
        };
        let mut msg = make_msg(cmd);
        msg.timeout_secs = 1;
        let result = execute_task(&msg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }
}
