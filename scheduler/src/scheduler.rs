use flowforge_common::config::AppConfig;
use flowforge_common::dag::{get_ready_tasks, validate_dag};
use flowforge_common::models::{DagDefinition, TaskMessage, TaskResult};
use flowforge_common::queue::TaskQueue;
use sqlx::PgPool;
use std::collections::HashSet;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct Scheduler {
    pool: PgPool,
    queue: TaskQueue,
    config: AppConfig,
}

impl Scheduler {
    pub fn new(pool: PgPool, queue: TaskQueue, config: AppConfig) -> Self {
        Self { pool, queue, config }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Scheduler loop started (interval={}s)", self.config.scheduler_interval_secs);

        loop {
            if let Err(e) = self.process_results().await {
                error!(error = %e, "Error processing results");
            }

            if let Err(e) = self.schedule_ready_tasks().await {
                error!(error = %e, "Error scheduling tasks");
            }

            if let Err(e) = self.check_stale_workers().await {
                error!(error = %e, "Error checking stale workers");
            }

            if let Err(e) = self.check_cron_schedules().await {
                error!(error = %e, "Error checking cron schedules");
            }

            metrics::counter!("scheduler.ticks").increment(1);
            tokio::time::sleep(tokio::time::Duration::from_secs(self.config.scheduler_interval_secs)).await;
        }
    }

    /// Process task results from workers.
    async fn process_results(&self) -> anyhow::Result<()> {
        loop {
            let result = self.queue.poll_result().await?;
            match result {
                Some(r) => self.handle_task_result(r).await?,
                None => break,
            }
        }
        Ok(())
    }

    async fn handle_task_result(&self, result: TaskResult) -> anyhow::Result<()> {
        info!(
            task_id = %result.task_id,
            dag_id = %result.dag_id,
            status = %result.status,
            attempt = result.attempt,
            "Processing task result"
        );

        // Update task instance in DB
        sqlx::query(
            "UPDATE task_instances SET status = $1, worker_id = $2, finished_at = NOW(), \
             output = $3, error = $4, attempt = $5 WHERE id = $6"
        )
        .bind(&result.status)
        .bind(&result.worker_id)
        .bind(&result.output)
        .bind(&result.error)
        .bind(result.attempt)
        .bind(result.task_instance_id)
        .execute(&self.pool)
        .await?;

        // If task failed and has retries left, re-enqueue
        if result.status == "failed" {
            let ti = sqlx::query_as::<_, flowforge_common::models::TaskInstance>(
                "SELECT * FROM task_instances WHERE id = $1"
            )
            .bind(result.task_instance_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(ti) = ti {
                if ti.attempt < ti.max_retries {
                    let new_attempt = ti.attempt + 1;
                    info!(task_id = %ti.task_id, attempt = new_attempt, "Retrying task");

                    sqlx::query("UPDATE task_instances SET status = 'retrying', attempt = $1 WHERE id = $2")
                        .bind(new_attempt)
                        .bind(ti.id)
                        .execute(&self.pool)
                        .await?;

                    // Exponential backoff delay
                    let delay_secs = 2u64.pow((new_attempt - 1) as u32).min(60);
                    let queue = self.queue.clone();
                    let task_msg = TaskMessage {
                        task_instance_id: ti.id,
                        run_id: ti.run_id,
                        dag_id: ti.dag_id.clone(),
                        task_id: ti.task_id.clone(),
                        command: ti.command.clone(),
                        attempt: new_attempt,
                        max_retries: ti.max_retries,
                        timeout_secs: 300,
                        env: Default::default(),
                    };

                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                        if let Err(e) = queue.enqueue_task(&task_msg).await {
                            error!(error = %e, "Failed to re-enqueue task for retry");
                        }
                    });

                    metrics::counter!("scheduler.task_retries").increment(1);
                    return Ok(());
                }
            }
        }

        // Check if all tasks for this run are complete
        self.check_run_completion(result.run_id).await?;

        metrics::counter!("scheduler.results_processed").increment(1);
        Ok(())
    }

    /// Check if a DAG run is fully complete and update its status.
    async fn check_run_completion(&self, run_id: Uuid) -> anyhow::Result<()> {
        let row: (i64, i64, i64) = sqlx::query_as(
            "SELECT \
                COUNT(*) FILTER (WHERE status IN ('pending','queued','running','retrying')), \
                COUNT(*) FILTER (WHERE status = 'success'), \
                COUNT(*) FILTER (WHERE status = 'failed' AND attempt >= max_retries) \
             FROM task_instances WHERE run_id = $1"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        let (in_progress, _succeeded, hard_failed) = row;

        if in_progress == 0 {
            let final_status = if hard_failed > 0 { "failed" } else { "success" };
            sqlx::query("UPDATE dag_runs SET status = $1, finished_at = NOW() WHERE id = $2")
                .bind(final_status)
                .bind(run_id)
                .execute(&self.pool)
                .await?;
            info!(run_id = %run_id, status = final_status, "DAG run completed");
            metrics::counter!("scheduler.runs_completed").increment(1);
        }

        Ok(())
    }

    /// Find tasks that are ready to execute and enqueue them.
    async fn schedule_ready_tasks(&self) -> anyhow::Result<()> {
        // Get all running DAG runs
        let runs: Vec<flowforge_common::models::DagRun> = sqlx::query_as(
            "SELECT * FROM dag_runs WHERE status IN ('pending', 'running')"
        )
        .fetch_all(&self.pool)
        .await?;

        for run in runs {
            // Mark as running if still pending
            if run.status == "pending" {
                sqlx::query("UPDATE dag_runs SET status = 'running', started_at = NOW() WHERE id = $1")
                    .bind(run.id)
                    .execute(&self.pool)
                    .await?;
            }

            // Fetch the DAG definition
            let dag_row = sqlx::query_as::<_, flowforge_common::models::Dag>(
                "SELECT * FROM dags WHERE dag_id = $1"
            )
            .bind(&run.dag_id)
            .fetch_optional(&self.pool)
            .await?;

            let dag_row = match dag_row {
                Some(d) => d,
                None => {
                    warn!(dag_id = %run.dag_id, "DAG not found, marking run as failed");
                    sqlx::query("UPDATE dag_runs SET status = 'failed', finished_at = NOW() WHERE id = $1")
                        .bind(run.id)
                        .execute(&self.pool)
                        .await?;
                    continue;
                }
            };

            let dag_def: DagDefinition = serde_json::from_value(dag_row.definition.clone())?;

            // Get completed and failed tasks for this run
            let completed_tasks: Vec<(String,)> = sqlx::query_as(
                "SELECT task_id FROM task_instances WHERE run_id = $1 AND status = 'success'"
            )
            .bind(run.id)
            .fetch_all(&self.pool)
            .await?;
            let completed: HashSet<String> = completed_tasks.into_iter().map(|r| r.0).collect();

            // Get tasks already queued/running
            let active_tasks: Vec<(String,)> = sqlx::query_as(
                "SELECT task_id FROM task_instances WHERE run_id = $1 AND status IN ('queued', 'running', 'retrying')"
            )
            .bind(run.id)
            .fetch_all(&self.pool)
            .await?;
            let active: HashSet<String> = active_tasks.into_iter().map(|r| r.0).collect();

            let ready = get_ready_tasks(&dag_def, &completed);

            for task_id in ready {
                if active.contains(&task_id) {
                    continue; // Already queued or running
                }

                let task_def = dag_def.tasks.iter().find(|t| t.id == task_id);
                let task_def = match task_def {
                    Some(t) => t,
                    None => continue,
                };

                let instance_id = Uuid::new_v4();
                let max_retries = task_def.retries.unwrap_or(dag_def.default_retries) as i32;

                // Insert task instance
                sqlx::query(
                    "INSERT INTO task_instances (id, run_id, task_id, dag_id, status, attempt, max_retries, command) \
                     VALUES ($1, $2, $3, $4, 'queued', 1, $5, $6) \
                     ON CONFLICT (run_id, task_id) DO NOTHING"
                )
                .bind(instance_id)
                .bind(run.id)
                .bind(&task_id)
                .bind(&run.dag_id)
                .bind(max_retries)
                .bind(&task_def.command)
                .execute(&self.pool)
                .await?;

                // Enqueue to Redis
                let msg = TaskMessage {
                    task_instance_id: instance_id,
                    run_id: run.id,
                    dag_id: run.dag_id.clone(),
                    task_id: task_id.clone(),
                    command: task_def.command.clone(),
                    attempt: 1,
                    max_retries,
                    timeout_secs: task_def.timeout_secs,
                    env: task_def.env.clone(),
                };

                self.queue.enqueue_task(&msg).await?;
                info!(task_id = %task_id, run_id = %run.id, "Task enqueued");
                metrics::counter!("scheduler.tasks_enqueued").increment(1);
            }
        }

        Ok(())
    }

    /// Detect workers that have stopped heartbeating and requeue their tasks.
    async fn check_stale_workers(&self) -> anyhow::Result<()> {
        let timeout = chrono::Utc::now()
            - chrono::Duration::seconds(self.config.heartbeat_timeout_secs as i64);

        let stale_workers: Vec<(String,)> = sqlx::query_as(
            "UPDATE worker_heartbeats SET is_alive = false \
             WHERE last_heartbeat < $1 AND is_alive = true \
             RETURNING worker_id"
        )
        .bind(timeout)
        .fetch_all(&self.pool)
        .await?;

        for (worker_id,) in &stale_workers {
            warn!(worker_id = %worker_id, "Worker marked as dead, requeuing tasks");

            let orphaned: Vec<flowforge_common::models::TaskInstance> = sqlx::query_as(
                "UPDATE task_instances SET status = 'queued', worker_id = NULL \
                 WHERE worker_id = $1 AND status = 'running' \
                 RETURNING *"
            )
            .bind(worker_id)
            .fetch_all(&self.pool)
            .await?;

            for ti in orphaned {
                let msg = TaskMessage {
                    task_instance_id: ti.id,
                    run_id: ti.run_id,
                    dag_id: ti.dag_id,
                    task_id: ti.task_id,
                    command: ti.command,
                    attempt: ti.attempt,
                    max_retries: ti.max_retries,
                    timeout_secs: 300,
                    env: Default::default(),
                };
                self.queue.enqueue_task(&msg).await?;
                metrics::counter!("scheduler.tasks_requeued").increment(1);
            }
        }

        Ok(())
    }

    /// Check cron-scheduled DAGs and trigger runs if due.
    async fn check_cron_schedules(&self) -> anyhow::Result<()> {
        let dags: Vec<flowforge_common::models::Dag> = sqlx::query_as(
            "SELECT * FROM dags WHERE schedule IS NOT NULL AND is_active = true"
        )
        .fetch_all(&self.pool)
        .await?;

        for dag in dags {
            if let Some(ref sched) = dag.schedule {
                let schedule = match sched.parse::<cron::Schedule>() {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(dag_id = %dag.dag_id, error = %e, "Invalid cron expression");
                        continue;
                    }
                };

                // Check if there's already a recent run
                let last_run: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
                    "SELECT created_at FROM dag_runs WHERE dag_id = $1 ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&dag.dag_id)
                .fetch_optional(&self.pool)
                .await?;

                let should_trigger = match last_run {
                    Some((last_created,)) => {
                        // Get next scheduled time after last run
                        schedule.after(&last_created).next().map_or(false, |next| {
                            next <= chrono::Utc::now()
                        })
                    }
                    None => {
                        // Never run — trigger if there's a valid next time
                        schedule.after(&chrono::Utc::now()).next().is_some()
                    }
                };

                if should_trigger {
                    let run_id = Uuid::new_v4();
                    sqlx::query(
                        "INSERT INTO dag_runs (id, dag_id, status, triggered_by) VALUES ($1, $2, 'pending', 'cron')"
                    )
                    .bind(run_id)
                    .bind(&dag.dag_id)
                    .execute(&self.pool)
                    .await?;
                    info!(dag_id = %dag.dag_id, run_id = %run_id, "Cron-triggered DAG run");
                    metrics::counter!("scheduler.cron_triggers").increment(1);
                }
            }
        }

        Ok(())
    }
}
