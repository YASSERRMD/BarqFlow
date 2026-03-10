CREATE TABLE IF NOT EXISTS secret_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(64) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(64) NOT NULL DEFAULT 'draft',
    last_validated_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_secret_providers_workspace_name
    ON secret_providers (workspace_id, LOWER(name));

CREATE INDEX IF NOT EXISTS idx_secret_providers_workspace_type
    ON secret_providers (workspace_id, provider_type);

CREATE TABLE IF NOT EXISTS workspace_policies (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    blocked_node_types JSONB NOT NULL DEFAULT '[]',
    blocked_support_tiers JSONB NOT NULL DEFAULT '[]',
    approval_required_node_types JSONB NOT NULL DEFAULT '[]',
    max_workflow_nodes INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS promotion_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    environment VARCHAR(64) NOT NULL,
    git_repo_url TEXT,
    git_branch VARCHAR(255),
    requires_approval BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_promotion_targets_workspace_name
    ON promotion_targets (workspace_id, LOWER(name));

CREATE INDEX IF NOT EXISTS idx_promotion_targets_workspace_environment
    ON promotion_targets (workspace_id, environment);

CREATE TABLE IF NOT EXISTS promotion_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES promotion_targets(id) ON DELETE CASCADE,
    requested_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(64) NOT NULL,
    source_control_ref VARCHAR(255),
    workflow_snapshot JSONB NOT NULL DEFAULT '{}',
    notes TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_promotion_requests_workspace_requested_at
    ON promotion_requests (workspace_id, requested_at DESC);

CREATE INDEX IF NOT EXISTS idx_promotion_requests_workspace_status
    ON promotion_requests (workspace_id, status);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_email VARCHAR(255),
    action VARCHAR(128) NOT NULL,
    resource_type VARCHAR(64) NOT NULL,
    resource_id UUID,
    summary TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_workspace_created_at
    ON audit_logs (workspace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_audit_logs_workspace_action
    ON audit_logs (workspace_id, action);
