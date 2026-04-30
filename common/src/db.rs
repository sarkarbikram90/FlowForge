use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

/// Create a PostgreSQL connection pool and run migrations.
pub async fn create_pool(database_url: &str) -> crate::error::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    info!("Connected to PostgreSQL");
    Ok(pool)
}

/// Run all pending migrations embedded in the binary.
pub async fn run_migrations(pool: &PgPool) -> crate::error::Result<()> {
    sqlx::query(SCHEMA_SQL).execute(pool).await?;
    info!("Database schema applied");
    Ok(())
}

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS dags (
    id UUID PRIMARY KEY,
    dag_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    schedule TEXT,
    default_retries INTEGER NOT NULL DEFAULT 3,
    definition JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dag_runs (
    id UUID PRIMARY KEY,
    dag_id TEXT NOT NULL REFERENCES dags(dag_id),
    status TEXT NOT NULL DEFAULT 'pending',
    triggered_by TEXT NOT NULL DEFAULT 'manual',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS task_instances (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES dag_runs(id),
    task_id TEXT NOT NULL,
    dag_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt INTEGER NOT NULL DEFAULT 1,
    max_retries INTEGER NOT NULL DEFAULT 3,
    command TEXT NOT NULL,
    worker_id TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    output TEXT,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(run_id, task_id)
);

CREATE TABLE IF NOT EXISTS worker_heartbeats (
    worker_id TEXT PRIMARY KEY,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    active_tasks INTEGER NOT NULL DEFAULT 0,
    is_alive BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_dag_runs_dag_id ON dag_runs(dag_id);
CREATE INDEX IF NOT EXISTS idx_dag_runs_status ON dag_runs(status);
CREATE INDEX IF NOT EXISTS idx_task_instances_run_id ON task_instances(run_id);
CREATE INDEX IF NOT EXISTS idx_task_instances_status ON task_instances(status);
"#;
