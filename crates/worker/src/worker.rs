use chrono::Utc;
use flowforge_common::{
    Result, TaskAttempt, TaskCompletionMessage, TaskDispatchMessage, TaskState, WorkerRegistration,
    WorkerStatus,
};
use flowforge_execution_engine::{ExecutionContext, ExecutorRegistry};
use flowforge_messaging::MessageBus;
use flowforge_persistence::Repository;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

pub struct WorkerAgent<R: Repository, M: MessageBus> {
    pub worker_id: String,
    pub hostname: String,
    pub repo: Arc<R>,
    pub bus: Arc<M>,
    pub executors: Arc<ExecutorRegistry>,
    pub max_concurrency: u32,
    pub current_load: Arc<AtomicU32>,
    pub capabilities: Vec<String>,
}

impl<R: Repository + 'static, M: MessageBus + 'static> WorkerAgent<R, M> {
    pub fn new(
        worker_id: &str,
        repo: Arc<R>,
        bus: Arc<M>,
        executors: Arc<ExecutorRegistry>,
        max_concurrency: u32,
    ) -> Self {
        let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown-host".to_string());
        let capabilities = vec![
            "shell".to_string(),
            "container".to_string(),
            "http".to_string(),
            "script".to_string(),
            "wait".to_string(),
            "condition".to_string(),
        ];

        Self {
            worker_id: worker_id.to_string(),
            hostname,
            repo,
            bus,
            executors,
            max_concurrency,
            current_load: Arc::new(AtomicU32::new(0)),
            capabilities,
        }
    }

    pub async fn register(&self) -> Result<()> {
        let reg = WorkerRegistration {
            worker_id: self.worker_id.clone(),
            hostname: self.hostname.clone(),
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            version: "0.2.0".to_string(),
            capabilities: self.capabilities.clone(),
            labels: HashMap::new(),
            max_concurrency: self.max_concurrency,
            current_load: 0,
            status: WorkerStatus::Online,
            first_registered_at: Utc::now(),
            last_heartbeat_at: Utc::now(),
        };

        info!(worker_id = %self.worker_id, "Registering worker agent");
        self.repo.register_worker(reg).await?;
        Ok(())
    }

    pub async fn run_heartbeat_loop(&self, cancel_token: CancellationToken) {
        info!(worker_id = %self.worker_id, "Starting worker heartbeat loop");
        while !cancel_token.is_cancelled() {
            let load = self.current_load.load(Ordering::SeqCst);
            if let Err(e) = self.repo.worker_heartbeat(&self.worker_id, load).await {
                error!(worker_id = %self.worker_id, "Failed to send heartbeat: {}", e);
            }

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_secs(10)) => {}
            }
        }
    }

    pub async fn run_task_pull_loop(self: Arc<Self>, cancel_token: CancellationToken) {
        info!(worker_id = %self.worker_id, "Starting task pull consumer loop");

        while !cancel_token.is_cancelled() {
            let load = self.current_load.load(Ordering::SeqCst);
            if load >= self.max_concurrency {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            match self.bus.pull_next_task(500).await {
                Ok(Some(msg)) => {
                    let worker = self.clone();
                    let child_cancel = cancel_token.clone();
                    tokio::spawn(async move {
                        worker.process_task(msg, child_cancel).await;
                    });
                }
                Ok(None) => {}
                Err(e) => {
                    error!(worker_id = %self.worker_id, "Error pulling task: {}", e);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    pub async fn process_task(&self, msg: TaskDispatchMessage, cancel_token: CancellationToken) {
        self.current_load.fetch_add(1, Ordering::SeqCst);
        let task_id = msg.task_id.clone();
        let task_run_id = msg.task_run_id;
        let attempt_id = Uuid::new_v4();
        let attempt_num = msg.attempt_number;

        info!(
            worker_id = %self.worker_id,
            task_id = %task_id,
            attempt = attempt_num,
            "Acquiring lease and beginning task execution"
        );

        // 1. Acquire task lease in DB (30 seconds)
        let _lease = match self
            .repo
            .acquire_or_renew_task_lease(task_run_id, &self.worker_id, attempt_id, 30)
            .await
        {
            Ok(l) => l,
            Err(e) => {
                error!(task_id = %task_id, "Failed to acquire task lease: {}", e);
                self.current_load.fetch_sub(1, Ordering::SeqCst);
                return;
            }
        };

        // 2. Create Task Attempt record
        let attempt = TaskAttempt {
            id: attempt_id,
            task_run_id,
            attempt_number: attempt_num,
            worker_id: self.worker_id.clone(),
            status: TaskState::Running,
            started_at: Utc::now(),
            finished_at: None,
            exit_code: None,
            stdout_log_path: None,
            stderr_log_path: None,
            error_message: None,
            duration_ms: None,
            created_at: Utc::now(),
        };
        let _ = self.repo.create_task_attempt(attempt).await;
        let _ = self
            .repo
            .update_task_run_status(
                task_run_id,
                TaskState::Running,
                Some(self.worker_id.clone()),
                None,
                None,
            )
            .await;

        // 3. Spawn background lease renewal task
        let lease_cancel = CancellationToken::new();
        let lease_cancel_clone = lease_cancel.clone();
        let repo_clone = self.repo.clone();
        let worker_id_clone = self.worker_id.clone();
        tokio::spawn(async move {
            while !lease_cancel_clone.is_cancelled() {
                tokio::select! {
                    _ = lease_cancel_clone.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        let _ = repo_clone.acquire_or_renew_task_lease(
                            task_run_id,
                            &worker_id_clone,
                            attempt_id,
                            30,
                        ).await;
                    }
                }
            }
        });

        // 4. Execute task through registered executor
        let ctx = ExecutionContext {
            task_run_id,
            attempt_id,
            worker_id: self.worker_id.clone(),
            message: msg.clone(),
        };

        let exec_result = match self.executors.get(&msg.task_type) {
            Ok(executor) => executor.execute(ctx, cancel_token).await,
            Err(e) => Err(e),
        };

        lease_cancel.cancel();

        // 5. Finalize state and publish completion
        let (final_status, output, error_msg, exit_code, duration_ms) = match exec_result {
            Ok(res) => (
                res.status,
                res.output,
                res.error,
                res.exit_code,
                res.duration.as_millis() as i64,
            ),
            Err(e) => (TaskState::Failed, None, Some(e.to_string()), None, 0),
        };

        let _ = self
            .repo
            .update_task_run_status(
                task_run_id,
                final_status,
                Some(self.worker_id.clone()),
                output.clone(),
                error_msg.clone(),
            )
            .await;

        let _ = self.repo.release_task_lease(task_run_id).await;

        let completion_msg = TaskCompletionMessage {
            task_run_id,
            attempt_id,
            worker_id: self.worker_id.clone(),
            status: final_status,
            exit_code,
            output_data: output,
            error_message: error_msg,
            duration_ms,
        };

        let _ = self.bus.publish_task_completion(&completion_msg).await;
        self.current_load.fetch_sub(1, Ordering::SeqCst);

        info!(
            worker_id = %self.worker_id,
            task_id = %task_id,
            status = ?final_status,
            "Task execution finished"
        );
    }
}
