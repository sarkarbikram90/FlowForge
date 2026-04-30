use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use flowforge_common::dag::validate_dag;
use flowforge_common::models::*;
use tracing::{error, info};
use uuid::Uuid;

use crate::routes::AppState;

// ─── Health & Metrics ───

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn metrics(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

// ─── DAG Endpoints ───

pub async fn list_dags(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Dag>>>, StatusCode> {
    let dags: Vec<Dag> = sqlx::query_as("SELECT * FROM dags ORDER BY created_at DESC")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to list DAGs");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse::ok(dags)))
}

pub async fn submit_dag(
    State(state): State<AppState>,
    Json(req): Json<DagSubmitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Dag>>), (StatusCode, Json<ApiResponse<()>>)> {
    let dag_def: DagDefinition = serde_yaml::from_str(&req.yaml).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(format!("Invalid YAML: {e}"))),
        )
    })?;

    // Validate DAG (cycles, missing deps, etc.)
    validate_dag(&dag_def).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(format!("DAG validation failed: {e}"))),
        )
    })?;

    let definition = serde_json::to_value(&dag_def).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(format!("Serialization error: {e}"))),
        )
    })?;

    let id = Uuid::new_v4();
    let dag = sqlx::query_as::<_, Dag>(
        "INSERT INTO dags (id, dag_id, name, description, schedule, default_retries, definition) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (dag_id) DO UPDATE SET \
            name = EXCLUDED.name, \
            description = EXCLUDED.description, \
            schedule = EXCLUDED.schedule, \
            default_retries = EXCLUDED.default_retries, \
            definition = EXCLUDED.definition, \
            updated_at = NOW() \
         RETURNING *"
    )
    .bind(id)
    .bind(&dag_def.id)
    .bind(&dag_def.name)
    .bind(&dag_def.description)
    .bind(&dag_def.schedule)
    .bind(dag_def.default_retries as i32)
    .bind(&definition)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to insert DAG");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err(format!("Database error: {e}"))),
        )
    })?;

    info!(dag_id = %dag_def.id, "DAG submitted");
    metrics::counter!("api.dags_submitted").increment(1);
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(dag))))
}

pub async fn get_dag(
    State(state): State<AppState>,
    Path(dag_id): Path<String>,
) -> Result<Json<ApiResponse<Dag>>, (StatusCode, Json<ApiResponse<()>>)> {
    let dag: Option<Dag> = sqlx::query_as("SELECT * FROM dags WHERE dag_id = $1")
        .bind(&dag_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get DAG");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string())))
        })?;

    match dag {
        Some(d) => Ok(Json(ApiResponse::ok(d))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err(format!("DAG '{dag_id}' not found"))))),
    }
}

pub async fn delete_dag(
    State(state): State<AppState>,
    Path(dag_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let result = sqlx::query("UPDATE dags SET is_active = false WHERE dag_id = $1")
        .bind(&dag_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to delete DAG");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string())))
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(ApiResponse::err(format!("DAG '{dag_id}' not found")))));
    }

    info!(dag_id = %dag_id, "DAG deactivated");
    Ok(Json(ApiResponse::ok(format!("DAG '{dag_id}' deactivated"))))
}

// ─── Run Endpoints ───

pub async fn trigger_run(
    State(state): State<AppState>,
    Json(req): Json<TriggerRunRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DagRun>>), (StatusCode, Json<ApiResponse<()>>)> {
    // Verify DAG exists and is active
    let dag: Option<Dag> = sqlx::query_as("SELECT * FROM dags WHERE dag_id = $1 AND is_active = true")
        .bind(&req.dag_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to check DAG");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string())))
        })?;

    if dag.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err(format!("Active DAG '{}' not found", req.dag_id))),
        ));
    }

    let run_id = Uuid::new_v4();
    let run = sqlx::query_as::<_, DagRun>(
        "INSERT INTO dag_runs (id, dag_id, status, triggered_by) VALUES ($1, $2, 'pending', $3) RETURNING *"
    )
    .bind(run_id)
    .bind(&req.dag_id)
    .bind(&req.triggered_by)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to create run");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string())))
    })?;

    info!(dag_id = %req.dag_id, run_id = %run_id, "DAG run triggered");
    metrics::counter!("api.runs_triggered").increment(1);
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(run))))
}

pub async fn list_runs(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<DagRun>>>, StatusCode> {
    let runs: Vec<DagRun> = sqlx::query_as(
        "SELECT * FROM dag_runs ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to list runs");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse::ok(runs)))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<ApiResponse<DagRun>>, (StatusCode, Json<ApiResponse<()>>)> {
    let run: Option<DagRun> = sqlx::query_as("SELECT * FROM dag_runs WHERE id = $1")
        .bind(run_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get run");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err(e.to_string())))
        })?;

    match run {
        Some(r) => Ok(Json(ApiResponse::ok(r))),
        None => Err((StatusCode::NOT_FOUND, Json(ApiResponse::err("Run not found")))),
    }
}

pub async fn get_run_tasks(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<TaskInstance>>>, StatusCode> {
    let tasks: Vec<TaskInstance> = sqlx::query_as(
        "SELECT * FROM task_instances WHERE run_id = $1 ORDER BY created_at"
    )
    .bind(run_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to get tasks");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse::ok(tasks)))
}

// ─── System Endpoints ───

pub async fn system_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<SystemStatus>>, StatusCode> {
    let (active_dags,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dags WHERE is_active = true")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (total_runs,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dag_runs")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (running_tasks,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM task_instances WHERE status IN ('queued', 'running')"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (active_workers,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM worker_heartbeats WHERE is_alive = true"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let queue_depth = state.queue.queue_depth().await.unwrap_or(0);

    Ok(Json(ApiResponse::ok(SystemStatus {
        active_dags,
        total_runs,
        running_tasks: running_tasks + queue_depth,
        active_workers,
        scheduler_healthy: true,
    })))
}

pub async fn list_workers(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<WorkerHeartbeat>>>, StatusCode> {
    let workers: Vec<(String, chrono::DateTime<chrono::Utc>, i32)> = sqlx::query_as(
        "SELECT worker_id, last_heartbeat, active_tasks FROM worker_heartbeats WHERE is_alive = true"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<WorkerHeartbeat> = workers
        .into_iter()
        .map(|(wid, ts, _)| WorkerHeartbeat {
            worker_id: wid,
            timestamp: ts,
            active_tasks: vec![],
        })
        .collect();

    Ok(Json(ApiResponse::ok(result)))
}
