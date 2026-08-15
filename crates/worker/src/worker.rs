use flowforge_common::{
    Result, TaskAttempt, TaskCompletionMessage, TaskDispatchMessage, TaskState,
    WorkerRegistration, WorkerStatus,
};
use flowforge_execution_engine::{ExecutionContext, ExecutorRegistry};
use flowforge_messaging::MessageBus;
use flowforge_observability::MetricsRegistry;
use flowforge_persistence::Repository;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct WorkerAgent<R: Repository, B: MessageBus> {
    worker_id: String,
    repo: Arc<R>,
    bus: Arc<B>,
    executors: Arc<ExecutorRegistry>,
    max_concurrency: usize,
    active_tasks: Arc<AtomicU32>,
    is_draining: Arc<AtomicBool>,
}

impl<R: Repository + 'static, B: MessageBus + 'static> WorkerAgent<R, B> {
    pub fn new(
        worker_id: &str,
        repo: Arc<R>,
        bus: Arc<B>,
        executors: Arc<ExecutorRegistry>,
        max_concurrency: usize,
    ) -> Self {
        Self {
            worker_id: worker_id.to_string(),
            repo,
            bus,
            executors,
            max_concurrency,
            active_tasks: Arc::new(AtomicU32::new(0)),
            is_draining: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    pub fn is_draining(&self) -> bool {
        self.is_draining.load(Ordering::SeqCst)
    }

    pub fn drain(&self) {
        info!(worker_id = %self.worker_id, "Worker draining initiated");
        self.is_draining.store(true, Ordering::SeqCst);
    }

    pub async fn register(&self) -> Result<()> {
        let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "localhost".to_string());
        let os = whoami::platform().to_string();
        let arch = whoami::arch().to_string();

        let mut capabilities = vec![
            "shell".to_string(),
            "http".to_string(),
            "script".to_string(),
            "wait".to_string(),
            "condition".to_string(),
        ];

        // Check if docker is available
        if std::process::Command::new("docker").arg("--version").output().is_ok() {
            capabilities.push("docker".to_string());
            capabilities.push("container".to_string());
        }

        let mut labels = HashMap::new();
        labels.insert("worker_group".to_string(), "general".to_string());
        labels.insert("os".to_string(), os.clone());

        let reg = WorkerRegistration {
            worker_id: self.worker_id.clone(),
            hostname,
            os,
            architecture: arch,
            version: "0.2.0".to_string(),
            capabilities,
            labels,
            max_concurrency: self.max_concurrency as u32,
            current_load: 0,
            status: WorkerStatus::Online,
            first_registered_at: chrono::Utc::now(),
            last_heartbeat_at: chrono::Utc::now(),
        };

        self.repo.register_worker(reg).await?;
        info!(worker_id = %self.worker_id, "Worker registered successfully");
        Ok(())
    }

    pub async fn run_heartbeat_loop(&self, cancel_token: CancellationToken) {
        while !cancel_token.is_cancelled() {
            let load = self.active_tasks.load(Ordering::SeqCst);
            if let Err(e) = self.repo.worker_heartbeat(&self.worker_id, load).await {
                warn!(worker_id = %self.worker_id, error = %e, "Worker heartbeat failed");
            }
            MetricsRegistry::set_worker_capacity(self.max_concurrency as f64);
            MetricsRegistry::set_worker_utilization(load as f64);

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            }
        }
    }

    pub async fn run_task_pull_loop(&self, cancel_token: CancellationToken) {
        info!(worker_id = %self.worker_id, "Starting task pull consumer loop");

        while !cancel_token.is_cancelled() && !self.is_draining.load(Ordering::SeqCst) {
            let current_load = self.active_tasks.load(Ordering::SeqCst) as usize;
            if current_load >= self.max_concurrency {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            match self.bus.pull_next_task(500).await {
                Ok(Some(msg)) => {
                    let agent = self.clone_self();
                    let task_token = cancel_token.clone();
                    tokio::spawn(async move {
                        agent.execute_task_wrapper(msg, task_token).await;
                    });
                }
                Ok(None) => {}
                Err(e) => {
                    error!(error = %e, "Error pulling task from message bus");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        info!(worker_id = %self.worker_id, "Task pull consumer stopped");
    }

    fn clone_self(&self) -> Self {
        Self {
            worker_id: self.worker_id.clone(),
            repo: self.repo.clone(),
            bus: self.bus.clone(),
            executors: self.executors.clone(),
            max_concurrency: self.max_concurrency,
            active_tasks: self.active_tasks.clone(),
            is_draining: self.is_draining.clone(),
        }
    }

    async fn execute_task_wrapper(&self, msg: TaskDispatchMessage, cancel_token: CancellationToken) {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        let attempt_id = Uuid::new_v4();

        info!(
            worker_id = %self.worker_id,
            task_id = %msg.task_id,
            task_run_id = %msg.task_run_id,
            "Acquiring lease and starting task execution"
        );

        // 1. Acquire task lease (with 30s timeout)
        let lease = match self
            .repo
            .acquire_or_renew_task_lease(msg.task_run_id, &self.worker_id, attempt_id, 30)
            .await
        {
            Ok(l) => l,
            Err(e) => {
                error!(task_id = %msg.task_id, error = %e, "Failed to acquire task lease, skipping execution");
                self.active_tasks.fetch_sub(1, Ordering::SeqCst);
                return;
            }
        };

        // 2. Transition state to RUNNING
        let _ = self
            .repo
            .update_task_run_status(
                msg.task_run_id,
                TaskState::Running,
                Some(self.worker_id.clone()),
                None,
                None,
            )
            .await;

        // 3. Create TaskAttempt record
        let start_time = chrono::Utc::now();
        let attempt = TaskAttempt {
            id: attempt_id,
            task_run_id: msg.task_run_id,
            attempt_number: msg.attempt_number,
            worker_id: self.worker_id.clone(),
            status: TaskState::Running,
            started_at: start_time,
            finished_at: None,
            exit_code: None,
            stdout_log_path: None,
            stderr_log_path: None,
            error_message: None,
            duration_ms: None,
            created_at: start_time,
        };
        let _ = self.repo.create_task_attempt(attempt).await;

        // 4. Background task lease renewal during execution
        let lease_renew_token = CancellationToken::new();
        let renew_token_clone = lease_renew_token.clone();
        let repo_clone = self.repo.clone();
        let worker_id_clone = self.worker_id.clone();
        let task_run_id = msg.task_run_id;

        tokio::spawn(async move {
            while !renew_token_clone.is_cancelled() {
                tokio::select! {
                    _ = renew_token_clone.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        let _ = repo_clone.acquire_or_renew_task_lease(task_run_id, &worker_id_clone, attempt_id, 30).await;
                    }
                }
            }
        });

        // 5. Execute task through executor registry
        let executor_res = self.executors.get(&msg.task_type);
        let exec_result = match executor_res {
            Ok(exec) => {
                let ctx = ExecutionContext {
                    task_run_id: msg.task_run_id,
                    attempt_id,
                    worker_id: self.worker_id.clone(),
                    message: msg.clone(),
                };
                exec.execute(ctx, cancel_token).await
            }
            Err(e) => Err(e),
        };

        // Stop lease renewal loop
        lease_renew_token.cancel();

        // 6. Record completion state
        let end_time = chrono::Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds();

        let (final_status, exit_code, output_data, error_message) = match exec_result {
            Ok(res) => (res.status, res.exit_code, res.output, res.error),
            Err(e) => (
                TaskState::Failed,
                Some(-1),
                None,
                Some(format!("Executor error: {}", e)),
            ),
        };

        info!(
            task_id = %msg.task_id,
            status = %final_status,
            duration_ms,
            "Task execution finished"
        );

        MetricsRegistry::record_task_executed(
            &msg.task_type,
            &final_status.to_string(),
            duration_ms as f64 / 1000.0,
        );

        // Update task run & attempt in database
        let _ = self
            .repo
            .update_task_run_status(
                msg.task_run_id,
                final_status,
                Some(self.worker_id.clone()),
                output_data.clone(),
                error_message.clone(),
            )
            .await;

        // Release lease
        let _ = self.repo.release_task_lease(msg.task_run_id).await;

        // Publish completion message
        let comp_msg = TaskCompletionMessage {
            task_run_id: msg.task_run_id,
            attempt_id,
            worker_id: self.worker_id.clone(),
            status: final_status,
            exit_code,
            output_data,
            error_message,
            duration_ms,
        };
        let _ = self.bus.publish_task_completion(&comp_msg).await;

        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
    }
}
