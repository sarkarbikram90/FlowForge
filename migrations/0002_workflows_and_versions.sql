-- FlowForge Migration: 0002_workflows_and_versions
-- Workflow Definitions, Immutable Versions, Tasks, Triggers and Variables

CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    concurrency_limit INT DEFAULT 10,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_workflows_project_name UNIQUE (project_id, name)
);

CREATE TABLE IF NOT EXISTS workflow_versions (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    version_number INT NOT NULL,
    definition_yaml TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    hash_sha256 VARCHAR(64) NOT NULL,
    is_latest BOOLEAN NOT NULL DEFAULT FALSE,
    change_summary TEXT,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_workflow_versions_num UNIQUE (workflow_id, version_number)
);

CREATE TABLE IF NOT EXISTS workflow_tasks (
    id UUID PRIMARY KEY,
    workflow_version_id UUID NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    task_id VARCHAR(100) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    command TEXT,
    image TEXT,
    depends_on TEXT[] NOT NULL DEFAULT '{}',
    retry_policy JSONB NOT NULL DEFAULT '{"max_attempts": 3, "backoff": "exponential", "initial_secs": 5, "max_secs": 300, "jitter": 0.2}',
    timeout_secs INT NOT NULL DEFAULT 300,
    env JSONB NOT NULL DEFAULT '{}',
    resource_limits JSONB NOT NULL DEFAULT '{}',
    CONSTRAINT uq_workflow_tasks_ver_task UNIQUE (workflow_version_id, task_id)
);

CREATE TABLE IF NOT EXISTS workflow_triggers (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    trigger_type VARCHAR(50) NOT NULL, -- cron, webhook, event, interval
    cron_expression VARCHAR(100),
    webhook_path VARCHAR(255),
    event_pattern JSONB,
    interval_secs INT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_fired_at TIMESTAMPTZ,
    next_fire_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_variables (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_workflow_variables_proj_key UNIQUE (project_id, key)
);

CREATE INDEX IF NOT EXISTS idx_workflows_org_proj ON workflows(organization_id, project_id);
CREATE INDEX IF NOT EXISTS idx_workflow_versions_wf ON workflow_versions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_triggers_next ON workflow_triggers(next_fire_at) WHERE is_active = TRUE;
