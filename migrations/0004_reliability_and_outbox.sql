-- FlowForge Migration: 0004_reliability_and_outbox
-- Transactional Outbox, Execution Events, Dead Letter Queue, Schedules

CREATE TABLE IF NOT EXISTS outbox_messages (
    id UUID PRIMARY KEY,
    organization_id UUID,
    project_id UUID,
    topic VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING', -- PENDING, PUBLISHED, FAILED
    retry_count INT NOT NULL DEFAULT 0,
    published_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS execution_events (
    id UUID PRIMARY KEY,
    workflow_run_id UUID REFERENCES workflow_runs(id) ON DELETE CASCADE,
    task_run_id UUID REFERENCES task_runs(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL DEFAULT 'system',
    data JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dead_letter_tasks (
    id UUID PRIMARY KEY,
    workflow_run_id UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    task_run_id UUID NOT NULL REFERENCES task_runs(id) ON DELETE CASCADE,
    task_id VARCHAR(100) NOT NULL,
    failure_reason VARCHAR(255) NOT NULL,
    total_attempts INT NOT NULL,
    payload JSONB NOT NULL,
    last_error TEXT,
    is_resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    resolved_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS schedules (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    cron_expr VARCHAR(100) NOT NULL,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    next_fire_at TIMESTAMPTZ,
    last_fired_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_outbox_pending ON outbox_messages(status, created_at) WHERE status = 'PENDING';
CREATE INDEX IF NOT EXISTS idx_exec_events_run ON execution_events(workflow_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_dlq_resolved ON dead_letter_tasks(is_resolved, created_at);
CREATE INDEX IF NOT EXISTS idx_schedules_next ON schedules(next_fire_at) WHERE is_enabled = TRUE;
