mod executor;

use flowforge_common::config::AppConfig;
use flowforge_common::db;
use flowforge_common::models::{TaskMessage, TaskResult};
use flowforge_common::queue::TaskQueue;
use sqlx::PgPool;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .json()
        .init();

    let worker_id = std::env::var("WORKER_ID")
        .unwrap_or_else(|_| format!("worker-{}", &Uuid::new_v4().to_string()[..8]));

    info!(worker_id = %worker_id, "FlowForge Worker starting...");
    let config = AppConfig::from_env();

    let pool = db::create_pool(&config.database_url).await?;
    let queue = TaskQueue::new(&config.redis_url).await?;

    // Heartbeat loop
    let hb_pool = pool.clone();
    let hb_worker_id = worker_id.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = sqlx::query(
                "INSERT INTO worker_heartbeats (worker_id, last_heartbeat, is_alive) \
                 VALUES ($1, NOW(), true) \
                 ON CONFLICT (worker_id) DO UPDATE SET last_heartbeat = NOW(), is_alive = true"
            )
            .bind(&hb_worker_id)
            .execute(&hb_pool)
            .await
            {
                error!(error = %e, "Failed to send heartbeat");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    });

    // Main work loop — run concurrent task processors
    let concurrency = config.worker_concurrency;
    info!(concurrency, "Starting worker loop");

    let mut handles = Vec::new();
    for i in 0..concurrency {
        let q = queue.clone();
        let p = pool.clone();
        let wid = worker_id.clone();
        handles.push(tokio::spawn(async move {
            worker_loop(i, &wid, q, p).await;
        }));
    }

    for h in handles {
        h.await?;
    }

    Ok(())
}

async fn worker_loop(slot: usize, worker_id: &str, queue: TaskQueue, pool: PgPool) {
    info!(slot, "Worker slot started");
    loop {
        match queue.dequeue_task(5.0).await {
            Ok(Some(msg)) => {
                info!(
                    slot,
                    task_id = %msg.task_id,
                    dag_id = %msg.dag_id,
                    attempt = msg.attempt,
                    "Executing task"
                );

                // Mark as running in DB
                if let Err(e) = sqlx::query(
                    "UPDATE task_instances SET status = 'running', worker_id = $1, started_at = NOW() WHERE id = $2"
                )
                .bind(worker_id)
                .bind(msg.task_instance_id)
                .execute(&pool)
                .await
                {
                    error!(error = %e, "Failed to update task status to running");
                }

                let start = std::time::Instant::now();
                let exec_result = executor::execute_task(&msg).await;
                let duration_ms = start.elapsed().as_millis() as u64;

                let result = match exec_result {
                    Ok(output) => {
                        info!(task_id = %msg.task_id, duration_ms, "Task succeeded");
                        metrics::counter!("worker.tasks_succeeded").increment(1);
                        TaskResult {
                            task_instance_id: msg.task_instance_id,
                            run_id: msg.run_id,
                            dag_id: msg.dag_id.clone(),
                            task_id: msg.task_id.clone(),
                            status: "success".to_string(),
                            attempt: msg.attempt,
                            output: Some(output),
                            error: None,
                            worker_id: worker_id.to_string(),
                            duration_ms,
                        }
                    }
                    Err(err) => {
                        warn!(task_id = %msg.task_id, error = %err, duration_ms, "Task failed");
                        metrics::counter!("worker.tasks_failed").increment(1);
                        TaskResult {
                            task_instance_id: msg.task_instance_id,
                            run_id: msg.run_id,
                            dag_id: msg.dag_id.clone(),
                            task_id: msg.task_id.clone(),
                            status: "failed".to_string(),
                            attempt: msg.attempt,
                            output: None,
                            error: Some(err),
                            worker_id: worker_id.to_string(),
                            duration_ms,
                        }
                    }
                };

                // Publish result and ack
                if let Err(e) = queue.publish_result(&result).await {
                    error!(error = %e, "Failed to publish task result");
                }
                if let Err(e) = queue.ack_task(&msg).await {
                    error!(error = %e, "Failed to ack task");
                }

                metrics::counter!("worker.tasks_processed").increment(1);
            }
            Ok(None) => {
                // No work available, loop will retry
            }
            Err(e) => {
                error!(error = %e, slot, "Error dequeuing task");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}
