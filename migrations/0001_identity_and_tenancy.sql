-- FlowForge Migration: 0001_identity_and_tenancy
-- Identity, Tenancy, RBAC and Access Management

CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_projects_org_slug UNIQUE (organization_id, slug)
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'Viewer',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_users_org_email UNIQUE (organization_id, email)
);

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    permissions TEXT[] NOT NULL DEFAULT '{}',
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_roles_org_name UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS permissions (
    id VARCHAR(100) PRIMARY KEY,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS service_accounts (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_service_accounts_org_name UNIQUE (organization_id, name)
);

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(16) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'WorkflowOperator',
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_projects_org ON projects(organization_id);
CREATE INDEX IF NOT EXISTS idx_users_org ON users(organization_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_org ON api_keys(organization_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(key_prefix);
