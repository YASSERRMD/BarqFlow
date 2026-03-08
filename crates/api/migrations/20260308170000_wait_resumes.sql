CREATE TABLE IF NOT EXISTS wait_resumes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES executions(id) ON DELETE CASCADE,
    node_name VARCHAR(255) NOT NULL,
    resume_token VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    resumed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_wait_resumes_execution_id
    ON wait_resumes (execution_id);
