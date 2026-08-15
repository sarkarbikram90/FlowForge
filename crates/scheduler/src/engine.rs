use flowforge_common::{
    FlowForgeError, Result, TaskDispatchMessage, TaskRun, TaskSpec, TaskState,
    WorkflowRun, WorkflowSpec, WorkflowState,
};
use flowforge_messaging::{MessageBus, SubjectBuilder};
use flowforge_observability::MetricsRegistry;
use flowforge_persistence::Repository;
use flowforge_workflow_engine::{DagGraph, VariableInterpolator};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct SchedulerEngine<R: Repository, B: MessageBus> {
    repo: Arc<R>,
    bus: Arc<B>,
    interval: Duration,
}

impl<R: Repository + 'static, B: MessageBus + 'static> SchedulerEngine<R, B> {
    pub fn new(repo: Arc<R>, bus: Arc<B>, interval: Duration) -> Self {
        Self {
            repo,
            bus,
            interval,
        }
    }

    pub async fn run_loop<F: Fn() -> bool + Send + Sync + 'static>(
        &self,
        is_leader: F,
        cancel_token: CancellationToken,
    ) {
        info!("Starting Scheduler Engine progression loop");

        while !cancel_token.is_cancelled() {
            if is_leader() {
                if let Err(e) = self.progress_all_active_runs().await {
                    error!(error = %e, "Error progressing active workflow runs");
                }
            }

            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.interval) => {}
            }
        }

        info!("Scheduler Engine progression loop stopped");
    }

    pub async fn progress_all_active_runs(&self) -> Result<()> {
        let (_org, proj) = self.repo.get_or_create_default_org().await?;
        let active_runs = self.repo.list_workflow_runs(proj.id, 100).await?;

        for run in active_runs {
            if run.status == WorkflowState::Pending || run.status == WorkflowState::Running {
                self.progress_run(&run).await?;
            }
        }

        Ok(())
    }

    pub async fn progress_run(&self, run: &WorkflowRun) -> Result<()> {
        // 1. Fetch immutable version
        let version = self.repo.get_version(run.workflow_version_id).await?;
        let spec: WorkflowSpec = serde_json::from_value(version.definition_json.clone())
            .map_err(|e| FlowForgeError::Validation(e.to_string()))?;

        // 2. Build DAG
        let dag = DagGraph::build(&spec.spec.tasks)?;

        // 3. Fetch existing task runs
        let task_runs = self.repo.get_task_runs_for_workflow_run(run.id).await?;
        let task_status_map: HashMap<String, TaskState> = task_runs
            .iter()
            .map(|t| (t.task_id.clone(), t.status))
            .collect();

        // If workflow is Pending, move to Running
        if run.status == WorkflowState::Pending {
            self.repo
                .update_workflow_run_status(run.id, WorkflowState::Running, None)
                .await?;
            MetricsRegistry::record_workflow_run_started(&spec.metadata.name);
        }

        let mut all_succeeded = true;
        let mut any_failed = false;

        for task_spec in &spec.spec.tasks {
            let current_status = task_status_map
                .get(&task_spec.id)
                .copied()
                .unwrap_or(TaskState::Pending);

            match current_status {
                TaskState::Succeeded => {}
                TaskState::Failed | TaskState::TimedOut | TaskState::DeadLetter => {
                    all_succeeded = false;
                    any_failed = true;
                }
                TaskState::Running | TaskState::Dispatched | TaskState::RetryWait | TaskState::Lost => {
                    all_succeeded = false;
                }
                TaskState::Pending | TaskState::Blocked | TaskState::Ready => {
                    all_succeeded = false;

                    // Check if all prerequisite tasks are succeeded
                    let deps = dag.get_dependencies(&task_spec.id);
                    let can_start = deps.iter().all(|dep_id| {
                        task_status_map.get(dep_id) == Some(&TaskState::Succeeded)
                    });

                    if can_start {
                        self.dispatch_task(run, task_spec).await?;
                    }
                }
                TaskState::Canceled => {
                    all_succeeded = false;
                    any_failed = true;
                }
            }
        }

        // Terminal evaluation for workflow run
        if all_succeeded && !spec.spec.tasks.is_empty() {
            info!(run_id = %run.id, "All tasks succeeded, marking workflow run SUCCEEDED");
            self.repo
                .update_workflow_run_status(run.id, WorkflowState::Succeeded, None)
                .await?;
            MetricsRegistry::record_workflow_run_completed(&spec.metadata.name, "succeeded", 1.0);
        } else if any_failed {
            info!(run_id = %run.id, "Task failure encountered without retry, marking workflow run FAILED");
            self.repo
                .update_workflow_run_status(
                    run.id,
                    WorkflowState::Failed,
                    Some("One or more required tasks failed".to_string()),
                )
                .await?;
            MetricsRegistry::record_workflow_run_completed(&spec.metadata.name, "failed", 1.0);
        }

        Ok(())
    }

    async fn dispatch_task(&self, run: &WorkflowRun, task: &TaskSpec) -> Result<()> {
        let task_run_id = Uuid::new_v4();

        // 1. Create task run record
        let task_run = TaskRun {
            id: task_run_id,
            workflow_run_id: run.id,
            task_id: task.id.clone(),
            task_type: task.task_type.clone(),
            status: TaskState::Dispatched,
            attempt_count: 1,
            max_attempts: task.retries.as_ref().map(|r| r.max_attempts).unwrap_or(3),
            current_worker_id: None,
            started_at: None,
            finished_at: None,
            duration_ms: None,
            output_data: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.repo.create_task_run(task_run).await?;

        // 2. Interpolate parameters and env
        let empty_map = HashMap::new();
        let interpolated_cmd = task
            .command
            .as_ref()
            .map(|c| VariableInterpolator::interpolate(c, &empty_map, &task.env));

        // 3. Build dispatch message
        let msg = TaskDispatchMessage {
            organization_id: run.organization_id,
            project_id: run.project_id,
            workflow_id: run.workflow_id,
            workflow_run_id: run.id,
            task_run_id,
            task_id: task.id.clone(),
            task_type: task.task_type.clone(),
            attempt_number: 1,
            max_attempts: task.retries.as_ref().map(|r| r.max_attempts).unwrap_or(3),
            timeout_secs: task.timeout_secs,
            command: interpolated_cmd,
            script: task.script.clone(),
            image: task.image.clone(),
            url: task.url.clone(),
            method: task.method.clone(),
            headers: task.headers.clone(),
            body: task.body.clone(),
            env: task.env.clone(),
            wait_secs: task.wait_secs,
        };

        // 4. Publish to MessageBus
        info!(task_id = %task.id, run_id = %run.id, "Dispatching task to message bus");
        self.bus.publish_task_dispatch(&msg).await?;

        Ok(())
    }
}
