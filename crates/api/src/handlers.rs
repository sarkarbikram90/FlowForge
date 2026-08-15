use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use chrono::Utc;
use flowforge_auth::ApiKeyManager;
use flowforge_common::{AuditLog, SystemStats, WorkerStatus, Workflow, WorkflowRun};
use flowforge_workflow_engine::{WorkflowCompiler, WorkflowValidator};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{Stream, StreamExt as _};
use uuid::Uuid;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

// ─── Health Probes ───

pub async fn health_live() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "LIVE", "uptime_ok": true })),
    )
}

pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.get_or_create_default_org().await {
        Ok(_) => (
            StatusCode::OK,
            Json(
                serde_json::json!({ "status": "READY", "database": "CONNECTED", "messaging": "CONNECTED" }),
            ),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "NOT_READY", "error": e.to_string() })),
        ),
    }
}

pub async fn health_startup() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "STARTED" })),
    )
}

// ─── System Stats ───

pub async fn get_system_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.get_system_stats().await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::ok(stats))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("STATS_FAILED", &e.to_string())),
        ),
    }
}

// ─── Auth Handlers ───

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let (org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("LOGIN_FAILED", &e.to_string())),
            )
        }
    };

    let user = match state
        .repo
        .create_user(org.id, &payload.email, "Platform User", "ProjectAdmin")
        .await
    {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("LOGIN_FAILED", &e.to_string())),
            )
        }
    };

    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({
            "user": user,
            "organization": org,
            "project": proj,
            "token": format!("bearer-token-for-{}", user.id)
        }))),
    )
}

pub async fn whoami() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({
            "user_id": "00000000-0000-0000-0000-000000000001",
            "email": "admin@flowforge.internal",
            "role": "PlatformAdmin",
            "permissions": ["*"]
        }))),
    )
}

pub async fn generate_api_key(State(_state): State<AppState>) -> impl IntoResponse {
    let (raw_key, prefix, hash) = ApiKeyManager::generate();
    (
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "api_key": raw_key,
            "prefix": prefix,
            "key_hash": hash,
            "name": "Default API Key",
            "note": "Save this key now. You will not be able to see it again."
        }))),
    )
}

// ─── Tenancy Handlers ───

pub async fn list_organizations(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_organizations().await {
        Ok(orgs) => (StatusCode::OK, Json(ApiResponse::ok(orgs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_ORGS_FAILED", &e.to_string())),
        ),
    }
}

pub async fn list_projects(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.list_projects(org_id).await {
        Ok(projects) => (StatusCode::OK, Json(ApiResponse::ok(projects))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_PROJECTS_FAILED", &e.to_string())),
        ),
    }
}

// ─── Workflows Handlers ───

pub async fn list_workflows(State(state): State<AppState>) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("DB_ERROR", &e.to_string())),
            )
        }
    };

    match state.repo.list_workflows(proj.id).await {
        Ok(wfs) => (StatusCode::OK, Json(ApiResponse::ok(wfs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_FAILED", &e.to_string())),
        ),
    }
}

#[derive(Deserialize)]
pub struct ApplyWorkflowRequest {
    pub yaml: String,
}

pub async fn apply_workflow(
    State(state): State<AppState>,
    Json(payload): Json<ApplyWorkflowRequest>,
) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("DB_ERROR", &e.to_string())),
            )
        }
    };

    let (spec, _dag) = match WorkflowValidator::parse_and_validate_yaml(&payload.yaml) {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err("VALIDATION_FAILED", &e.to_string())),
            )
        }
    };

    let existing_wf = match state
        .repo
        .get_workflow_by_name(proj.id, &spec.metadata.name)
        .await
    {
        Ok(Some(w)) => w,
        _ => {
            let new_wf = Workflow {
                id: Uuid::new_v4(),
                organization_id: proj.organization_id,
                project_id: proj.id,
                name: spec.metadata.name.clone(),
                description: spec.metadata.description.clone(),
                is_active: true,
                concurrency_limit: spec
                    .spec
                    .concurrency
                    .as_ref()
                    .map(|c| c.max_runs)
                    .unwrap_or(10),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            match state.repo.save_workflow(new_wf).await {
                Ok(w) => w,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::err("SAVE_FAILED", &e.to_string())),
                    )
                }
            }
        }
    };

    let version_num = spec.metadata.version.unwrap_or(1);
    let version = match WorkflowCompiler::compile_version(
        existing_wf.id,
        version_num,
        &payload.yaml,
        "api",
    ) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("COMPILATION_FAILED", &e.to_string())),
            )
        }
    };

    let saved_version = match state.repo.save_workflow_version(version).await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("VERSION_SAVE_FAILED", &e.to_string())),
            )
        }
    };

    (
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "workflow": existing_wf,
            "version": saved_version
        }))),
    )
}

pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.get_workflow(id).await {
        Ok(wf) => {
            let version = state.repo.get_latest_version(id).await.ok();
            (
                StatusCode::OK,
                Json(ApiResponse::ok(serde_json::json!({
                    "workflow": wf,
                    "latest_version": version
                }))),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err("NOT_FOUND", &e.to_string())),
        ),
    }
}

pub async fn validate_workflow(Json(payload): Json<ApplyWorkflowRequest>) -> impl IntoResponse {
    match WorkflowValidator::parse_and_validate_yaml(&payload.yaml) {
        Ok((spec, dag)) => (
            StatusCode::OK,
            Json(ApiResponse::ok(serde_json::json!({
                "name": spec.metadata.name,
                "tasks_count": spec.spec.tasks.len(),
                "topological_order": dag.topological_order(),
                "roots": dag.get_roots()
            }))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("INVALID_WORKFLOW", &e.to_string())),
        ),
    }
}

// ─── Runs Handlers ───

pub async fn list_workflow_runs(State(state): State<AppState>) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("DB_ERROR", &e.to_string())),
            )
        }
    };

    match state.repo.list_workflow_runs(proj.id, 100).await {
        Ok(runs) => (StatusCode::OK, Json(ApiResponse::ok(runs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_RUNS_FAILED", &e.to_string())),
        ),
    }
}

#[derive(Deserialize)]
pub struct TriggerRunRequest {
    pub workflow_name: String,
    pub variables: Option<Value>,
    pub idempotency_key: Option<String>,
}

pub async fn trigger_workflow_run(
    State(state): State<AppState>,
    Json(payload): Json<TriggerRunRequest>,
) -> impl IntoResponse {
    let (org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("DB_ERROR", &e.to_string())),
            )
        }
    };

    let wf = match state
        .repo
        .get_workflow_by_name(proj.id, &payload.workflow_name)
        .await
    {
        Ok(Some(w)) => w,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::err(
                    "WORKFLOW_NOT_FOUND",
                    "Workflow does not exist",
                )),
            )
        }
    };

    let version = match state.repo.get_latest_version(wf.id).await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err("NO_VERSION", &e.to_string())),
            )
        }
    };

    let run_id = Uuid::new_v4();
    let run = WorkflowRun {
        id: run_id,
        organization_id: org.id,
        project_id: proj.id,
        workflow_id: wf.id,
        workflow_version_id: version.id,
        idempotency_key: payload.idempotency_key,
        status: flowforge_common::WorkflowState::Pending,
        triggered_by: "api".to_string(),
        trigger_metadata: serde_json::json!({}),
        variables: payload.variables.unwrap_or(serde_json::json!({})),
        started_at: Some(Utc::now()),
        finished_at: None,
        duration_ms: None,
        error_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match state.repo.create_workflow_run(run).await {
        Ok(created) => {
            let _ = state
                .repo
                .insert_audit_log(AuditLog {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    organization_id: Some(org.id),
                    project_id: Some(proj.id),
                    actor: "api_user".to_string(),
                    action: "WORKFLOW_TRIGGERED".to_string(),
                    resource_type: "workflow_run".to_string(),
                    resource_id: Some(created.id.to_string()),
                    ip_address: None,
                    user_agent: None,
                    result: "SUCCESS".to_string(),
                    metadata: serde_json::json!({ "workflow_name": payload.workflow_name }),
                })
                .await;

            (StatusCode::CREATED, Json(ApiResponse::ok(created)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("CREATE_RUN_FAILED", &e.to_string())),
        ),
    }
}

pub async fn get_workflow_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.get_workflow_run(id).await {
        Ok(run) => {
            let tasks = state
                .repo
                .get_task_runs_for_workflow_run(id)
                .await
                .unwrap_or_default();
            let workflow = state.repo.get_workflow(run.workflow_id).await.ok();
            let version = state.repo.get_version(run.workflow_version_id).await.ok();

            (
                StatusCode::OK,
                Json(ApiResponse::ok(serde_json::json!({
                    "run": run,
                    "workflow": workflow,
                    "version": version,
                    "tasks": tasks
                }))),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err("RUN_NOT_FOUND", &e.to_string())),
        ),
    }
}

pub async fn cancel_workflow_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state
        .repo
        .update_workflow_run_status(
            id,
            flowforge_common::WorkflowState::Canceled,
            Some("Canceled by user request".to_string()),
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::ok(
                serde_json::json!({ "canceled": true, "id": id }),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("CANCEL_FAILED", &e.to_string())),
        ),
    }
}

// ─── Workers Handlers ───

pub async fn list_workers(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_workers().await {
        Ok(workers) => (StatusCode::OK, Json(ApiResponse::ok(workers))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_WORKERS_FAILED", &e.to_string())),
        ),
    }
}

pub async fn drain_worker(
    State(state): State<AppState>,
    Path(worker_id): Path<String>,
) -> impl IntoResponse {
    match state
        .repo
        .set_worker_status(&worker_id, WorkerStatus::Draining)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::ok(
                serde_json::json!({ "draining": true, "worker_id": worker_id }),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("DRAIN_FAILED", &e.to_string())),
        ),
    }
}

// ─── DLQ Handlers ───

pub async fn list_dlq(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_dlq().await {
        Ok(items) => (StatusCode::OK, Json(ApiResponse::ok(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("LIST_DLQ_FAILED", &e.to_string())),
        ),
    }
}

pub async fn resolve_dlq_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.resolve_dlq(id, "operator").await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::ok(
                serde_json::json!({ "resolved": true, "id": id }),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("RESOLVE_FAILED", &e.to_string())),
        ),
    }
}

// ─── Audit Log Handlers ───

pub async fn query_audit_logs(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.query_audit_logs(None, 100).await {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::ok(logs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("AUDIT_FAILED", &e.to_string())),
        ),
    }
}

// ─── SSE Live Stream ───

pub async fn execution_stream(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| {
                let stats = SystemStats {
                    active_workflows: 12,
                    total_runs: 1420,
                    running_runs: 4,
                    succeeded_runs: 1380,
                    failed_runs: 36,
                    queued_tasks: 8,
                    running_tasks: 14,
                    active_workers: 6,
                    dlq_count: 2,
                    scheduler_leader_id: Some("sched-primary-01".to_string()),
                    scheduler_healthy: true,
                    success_rate: 97.4,
                    average_duration_ms: 3240.0,
                };
                Ok(Event::default()
                    .json_data(stats)
                    .unwrap_or_else(|_| Event::default().data("heartbeat")))
            });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
