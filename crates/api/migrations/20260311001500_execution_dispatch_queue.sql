CREATE TABLE IF NOT EXISTS execution_dispatch_queue (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL UNIQUE REFERENCES executions(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    queue_kind TEXT NOT NULL,
    source TEXT NOT NULL,
    status TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 100,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    claimed_at TIMESTAMPTZ,
    lease_expires_at TIMESTAMPTZ,
    worker_id TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    CHECK (queue_kind IN ('run', 'trigger')),
    CHECK (status IN ('queued', 'leased', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_execution_dispatch_queue_claim
    ON execution_dispatch_queue (queue_kind, status, available_at, priority, created_at);

CREATE INDEX IF NOT EXISTS idx_execution_dispatch_queue_workspace_status
    ON execution_dispatch_queue (workspace_id, status, queue_kind);

CREATE INDEX IF NOT EXISTS idx_execution_dispatch_queue_lease
    ON execution_dispatch_queue (status, lease_expires_at)
    WHERE status = 'leased';
