use async_trait::async_trait;
use flowforge_common::{FlowForgeError, Result, TaskDispatchMessage, TaskState};
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::info;
use crate::executor::{ExecutionContext, TaskExecutionResult, TaskExecutor};

pub struct HttpExecutor {
    client: Client,
}

impl Default for HttpExecutor {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl TaskExecutor for HttpExecutor {
    fn supported_type(&self) -> &str {
        "http"
    }

    async fn validate(&self, message: &TaskDispatchMessage) -> Result<()> {
        if message.url.is_none() {
            return Err(FlowForgeError::Validation(
                "HTTP task requires 'url'".to_string(),
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
        let url = ctx.message.url.as_deref().unwrap_or("");
        let method_str = ctx.message.method.as_deref().unwrap_or("GET").to_uppercase();

        info!(task_id = %ctx.message.task_id, method = %method_str, url = %url, "Executing HTTP task");

        let method = match method_str.as_str() {
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            "HEAD" => reqwest::Method::HEAD,
            _ => reqwest::Method::GET,
        };

        let mut req = self.client.request(method, url);
        for (k, v) in &ctx.message.headers {
            req = req.header(k, v);
        }
        if let Some(body) = &ctx.message.body {
            req = req.body(body.clone());
        }

        tokio::select! {
            _ = cancel_token.cancelled() => {
                Ok(TaskExecutionResult {
                    status: TaskState::Canceled,
                    exit_code: None,
                    output: None,
                    error: Some("HTTP task canceled".to_string()),
                    duration: start_time.elapsed(),
                })
            }
            res = req.send() => {
                let duration = start_time.elapsed();
                match res {
                    Ok(resp) => {
                        let status_code = resp.status().as_u16() as i32;
                        let is_success = resp.status().is_success();
                        let body_text = resp.text().await.unwrap_or_default();

                        if is_success {
                            Ok(TaskExecutionResult {
                                status: TaskState::Succeeded,
                                exit_code: Some(status_code),
                                output: Some(body_text),
                                error: None,
                                duration,
                            })
                        } else {
                            Ok(TaskExecutionResult {
                                status: TaskState::Failed,
                                exit_code: Some(status_code),
                                output: Some(body_text.clone()),
                                error: Some(format!("HTTP request failed with status code {}: {}", status_code, body_text)),
                                duration,
                            })
                        }
                    }
                    Err(e) => Ok(TaskExecutionResult {
                        status: TaskState::Failed,
                        exit_code: None,
                        output: None,
                        error: Some(format!("HTTP network error: {}", e)),
                        duration,
                    }),
                }
            }
        }
    }
}
