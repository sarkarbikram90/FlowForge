-- FlowForge Migration: 0005_security_and_notifications
-- Immutable Audit Logs, Secrets References, Notification Channels and Rules

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    actor VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    result VARCHAR(50) NOT NULL DEFAULT 'SUCCESS', -- SUCCESS, FAILURE, DENIED
    metadata JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS secret_references (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    provider VARCHAR(50) NOT NULL DEFAULT 'vault', -- vault, aws, gcp, azure, env
    external_key VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_secret_references_proj_name UNIQUE (project_id, name)
);

CREATE TABLE IF NOT EXISTS notification_channels (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    channel_type VARCHAR(50) NOT NULL, -- slack, email, webhook, pagerduty, teams
    name VARCHAR(255) NOT NULL,
    config JSONB NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notification_rules (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES notification_channels(id) ON DELETE CASCADE,
    event_types TEXT[] NOT NULL,
    severity VARCHAR(50) NOT NULL DEFAULT 'info',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS notification_deliveries (
    id UUID PRIMARY KEY,
    rule_id UUID REFERENCES notification_rules(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_org ON audit_logs(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
