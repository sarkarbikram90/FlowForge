-- FlowForge Migration: 0003_executions_and_leases
-- Workflow Runs, Task Runs, Attempts, Leases, Worker Registrations and Scheduler Leases

CREATE TABLE IF NOT EXISTS workflow_runs (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    workflow_version_id UUID NOT NULL REFERENCES workflow_versions(id),
    idempotency_key VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    triggered_by VARCHAR(255) NOT NULL DEFAULT 'manual',
    trigger_metadata JSONB NOT NULL DEFAULT '{}',
    variables JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    duration_ms BIGINT,
    error_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_workflow_runs_idem UNIQUE (project_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS task_runs (
    id UUID PRIMARY KEY,
    workflow_run_id UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    task_id VARCHAR(100) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    attempt_count INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    current_worker_id VARCHAR(255),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    duration_ms BIGINT,
    output_data TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_task_runs_run_task UNIQUE (workflow_run_id, task_id)
);

CREATE TABLE IF NOT EXISTS task_attempts (
    id UUID PRIMARY KEY,
    task_run_id UUID NOT NULL REFERENCES task_runs(id) ON DELETE CASCADE,
    attempt_number INT NOT NULL,
    worker_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    exit_code INT,
    stdout_log_path TEXT,
    stderr_log_path TEXT,
    error_message TEXT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_task_attempts_run_att UNIQUE (task_run_id, attempt_number)
);

CREATE TABLE IF NOT EXISTS task_leases (
    task_run_id UUID PRIMARY KEY REFERENCES task_runs(id) ON DELETE CASCADE,
    worker_id VARCHAR(255) NOT NULL,
    attempt_id UUID NOT NULL,
    lease_token VARCHAR(255) NOT NULL,
    lease_version BIGINT NOT NULL DEFAULT 1,
    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS worker_registrations (
    worker_id VARCHAR(255) PRIMARY KEY,
    hostname VARCHAR(255) NOT NULL,
    os VARCHAR(50) NOT NULL,
    architecture VARCHAR(50) NOT NULL,
    version VARCHAR(50) NOT NULL,
    capabilities TEXT[] NOT NULL DEFAULT '{}',
    labels JSONB NOT NULL DEFAULT '{}',
    max_concurrency INT NOT NULL DEFAULT 4,
    current_load INT NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'ONLINE',
    first_registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scheduler_leases (
    service_name VARCHAR(100) PRIMARY KEY,
    leader_id VARCHAR(255) NOT NULL,
    lease_version BIGINT NOT NULL DEFAULT 1,
    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_runs_proj_status ON workflow_runs(project_id, status);
CREATE INDEX IF NOT EXISTS idx_workflow_runs_wf ON workflow_runs(workflow_id);
CREATE INDEX IF NOT EXISTS idx_task_runs_run ON task_runs(workflow_run_id);
CREATE INDEX IF NOT EXISTS idx_task_runs_status ON task_runs(status);
CREATE INDEX IF NOT EXISTS idx_task_leases_expires ON task_leases(expires_at);
CREATE INDEX IF NOT EXISTS idx_worker_registrations_heartbeat ON worker_registrations(last_heartbeat_at);
