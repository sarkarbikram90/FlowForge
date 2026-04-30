use axum::{routing::get, routing::post, Router};
use flowforge_common::queue::TaskQueue;
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub queue: TaskQueue,
    pub metrics_handle: PrometheusHandle,
}

pub fn create_router(pool: PgPool, queue: TaskQueue, metrics_handle: PrometheusHandle) -> Router {
    let state = AppState {
        pool,
        queue,
        metrics_handle,
    };

    Router::new()
        // Health
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::metrics))
        // DAGs
        .route("/api/v1/dags", get(handlers::list_dags))
        .route("/api/v1/dags", post(handlers::submit_dag))
        .route("/api/v1/dags/{dag_id}", get(handlers::get_dag))
        .route("/api/v1/dags/{dag_id}", axum::routing::delete(handlers::delete_dag))
        // Runs
        .route("/api/v1/runs", post(handlers::trigger_run))
        .route("/api/v1/runs", get(handlers::list_runs))
        .route("/api/v1/runs/{run_id}", get(handlers::get_run))
        .route("/api/v1/runs/{run_id}/tasks", get(handlers::get_run_tasks))
        // System
        .route("/api/v1/status", get(handlers::system_status))
        .route("/api/v1/workers", get(handlers::list_workers))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
