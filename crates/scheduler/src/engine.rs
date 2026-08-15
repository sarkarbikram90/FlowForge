use chrono::Utc;
use flowforge_common::{Result, TaskDispatchMessage, TaskState, WorkflowState};
use flowforge_messaging::MessageBus;
use flowforge_persistence::Repository;
use flowforge_workflow_engine::{DagGraph, WorkflowValidator};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

pub struct SchedulerEngine<R: Repository, M: MessageBus> {
    repo: Arc<R>,
    bus: Arc<M>,
    poll_interval: Duration,
}

impl<R: Repository + 'static, M: MessageBus + 'static> SchedulerEngine<R, M> {
    pub fn new(repo: Arc<R>, bus: Arc<M>, poll_interval: Duration) -> Self {
        Self {
            repo,
            bus,
            poll_interval,
        }
    }

    pub async fn run_progression_loop(&self, cancel_token: CancellationToken) {
        self.run_loop(|| true, cancel_token).await;
    }

    pub async fn run_loop<F>(&self, is_leader: F, cancel_token: CancellationToken)
    where
        F: Fn() -> bool + Send + 'static,
    {
        info!("Starting Scheduler progression loop");
        while !cancel_token.is_cancelled() {
            if is_leader() {
                if let Err(e) = self.progress_all_active_runs().await {
                    error!("Error in scheduler progression cycle: {}", e);
                }
            }
            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = tokio::time::sleep(self.poll_interval) => {}
            }
        }
        info!("Scheduler progression loop stopped");
    }

    pub async fn progress_all_active_runs(&self) -> Result<()> {
        let (org, proj) = self.repo.get_or_create_default_org().await?;
        let active_runs = self.repo.list_workflow_runs(proj.id, 50).await?;

        for run in active_runs {
            if run.status == WorkflowState::Pending || run.status == WorkflowState::Running {
                self.progress_single_run(run.id, org.id, proj.id).await?;
            }
        }
        Ok(())
    }

    pub async fn progress_single_run(
        &self,
        run_id: Uuid,
        org_id: Uuid,
        proj_id: Uuid,
    ) -> Result<()> {
        let run = self.repo.get_workflow_run(run_id).await?;
        let version = self.repo.get_version(run.workflow_version_id).await?;
        let (spec, _dag) = WorkflowValidator::parse_and_validate_yaml(&version.definition_yaml)?;
        let dag = DagGraph::build(&spec.spec.tasks)?;

        let existing_task_runs = self.repo.get_task_runs_for_workflow_run(run_id).await?;
        let mut task_states = std::collections::HashMap::new();
        for tr in &existing_task_runs {
            task_states.insert(tr.task_id.clone(), tr.status);
        }

        // If run is still Pending, transition to Running
        if run.status == WorkflowState::Pending {
            self.repo
                .update_workflow_run_status(run_id, WorkflowState::Running, None)
                .await?;
        }

        let mut all_completed = true;
        let mut any_failed = false;

        for task in &spec.spec.tasks {
            let current_state = task_states
                .get(&task.id)
                .copied()
                .unwrap_or(TaskState::Pending);

            match current_state {
                TaskState::Succeeded => continue,
                TaskState::Failed
                | TaskState::TimedOut
                | TaskState::Canceled
                | TaskState::DeadLetter => {
                    any_failed = true;
                }
                TaskState::Running | TaskState::Dispatched => {
                    all_completed = false;
                }
                TaskState::Pending | TaskState::Blocked | TaskState::Ready => {
                    all_completed = false;
                    // Evaluate dependencies
                    let deps = dag.get_dependencies(&task.id);
                    let mut ready = true;
                    for dep in deps {
                        if task_states.get(&dep).copied() != Some(TaskState::Succeeded) {
                            ready = false;
                            break;
                        }
                    }

                    if ready {
                        // Dispatch task
                        let task_run_id =
                            match existing_task_runs.iter().find(|t| t.task_id == task.id) {
                                Some(tr) => tr.id,
                                None => {
                                    let new_tr = flowforge_common::TaskRun {
                                        id: Uuid::new_v4(),
                                        workflow_run_id: run_id,
                                        task_id: task.id.clone(),
                                        task_type: task.task_type.clone(),
                                        status: TaskState::Ready,
                                        attempt_count: 0,
                                        max_attempts: task
                                            .retries
                                            .as_ref()
                                            .map(|r| r.max_attempts)
                                            .unwrap_or(3),
                                        current_worker_id: None,
                                        started_at: None,
                                        finished_at: None,
                                        duration_ms: None,
                                        output_data: None,
                                        error_message: None,
                                        created_at: Utc::now(),
                                        updated_at: Utc::now(),
                                    };
                                    self.repo.create_task_run(new_tr).await?.id
                                }
                            };

                        let msg = TaskDispatchMessage {
                            organization_id: org_id,
                            project_id: proj_id,
                            workflow_id: run.workflow_id,
                            workflow_run_id: run_id,
                            task_run_id,
                            task_id: task.id.clone(),
                            task_type: task.task_type.clone(),
                            attempt_number: 1,
                            max_attempts: task
                                .retries
                                .as_ref()
                                .map(|r| r.max_attempts)
                                .unwrap_or(3),
                            timeout_secs: task.timeout_secs,
                            command: task.command.clone(),
                            script: task.script.clone(),
                            image: task.image.clone(),
                            url: task.url.clone(),
                            method: task.method.clone(),
                            headers: task.headers.clone(),
                            body: task.body.clone(),
                            env: task.env.clone(),
                            wait_secs: task.wait_secs,
                        };

                        info!(run_id = %run_id, task_id = %task.id, "Dispatching ready task to message bus");
                        self.bus.publish_task_dispatch(&msg).await?;
                        self.repo
                            .update_task_run_status(
                                task_run_id,
                                TaskState::Dispatched,
                                None,
                                None,
                                None,
                            )
                            .await?;
                    }
                }
                TaskState::RetryWait | TaskState::Lost => {
                    all_completed = false;
                }
            }
        }

        if all_completed && !any_failed {
            info!(run_id = %run_id, "All tasks succeeded. Finalizing workflow run as SUCCEEDED");
            self.repo
                .update_workflow_run_status(run_id, WorkflowState::Succeeded, None)
                .await?;
        } else if any_failed {
            info!(run_id = %run_id, "Workflow contains unrecoverable failed task. Finalizing as FAILED");
            self.repo
                .update_workflow_run_status(
                    run_id,
                    WorkflowState::Failed,
                    Some("Task execution failed".to_string()),
                )
                .await?;
        }

        Ok(())
    }
}
