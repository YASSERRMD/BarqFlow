CREATE TABLE IF NOT EXISTS execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES executions(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    level VARCHAR(32) NOT NULL,
    event_type VARCHAR(64),
    message TEXT NOT NULL,
    node_id VARCHAR(255),
    node_name VARCHAR(255),
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_execution_logs_execution_id_created_at
    ON execution_logs (execution_id, created_at);

CREATE INDEX IF NOT EXISTS idx_execution_logs_workflow_id_created_at
    ON execution_logs (workflow_id, created_at);
