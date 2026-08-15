-- FlowForge Migration: 0006_usage_and_quotas
-- Resource Quotas and Multi-Tenant Usage Accounting

CREATE TABLE IF NOT EXISTS quotas (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    max_concurrent_runs INT NOT NULL DEFAULT 100,
    max_concurrent_tasks INT NOT NULL DEFAULT 500,
    max_workflows INT NOT NULL DEFAULT 1000,
    max_workers INT NOT NULL DEFAULT 50,
    retention_days INT NOT NULL DEFAULT 90,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    period_date DATE NOT NULL,
    total_runs BIGINT NOT NULL DEFAULT 0,
    total_tasks BIGINT NOT NULL DEFAULT 0,
    compute_seconds BIGINT NOT NULL DEFAULT 0,
    storage_bytes BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_usage_proj_period UNIQUE (project_id, period_date)
);

CREATE INDEX IF NOT EXISTS idx_usage_org_period ON usage_records(organization_id, period_date DESC);
