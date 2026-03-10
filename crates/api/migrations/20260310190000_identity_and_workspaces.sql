CREATE TABLE IF NOT EXISTS workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    created_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workspace_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (workspace_id, user_id)
);

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_prefix VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_workspace_memberships_user_id ON workspace_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_workspace_memberships_workspace_id ON workspace_memberships(workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_workspace_id ON api_keys(workspace_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);

ALTER TABLE users ADD COLUMN IF NOT EXISTS active_workspace_id UUID;
ALTER TABLE workflows ADD COLUMN IF NOT EXISTS workspace_id UUID;
ALTER TABLE workflows ADD COLUMN IF NOT EXISTS owner_user_id UUID;
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS workspace_id UUID;
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS owner_user_id UUID;
ALTER TABLE tags ADD COLUMN IF NOT EXISTS workspace_id UUID;

INSERT INTO workspaces (id, name, slug, created_by_user_id, created_at, updated_at)
SELECT
    gen_random_uuid(),
    COALESCE(NULLIF(split_part(u.email, '@', 1), ''), 'workspace') || ' Workspace',
    lower(regexp_replace(COALESCE(NULLIF(split_part(u.email, '@', 1), ''), 'workspace'), '[^a-zA-Z0-9]+', '-', 'g'))
        || '-' || substring(replace(u.id::text, '-', '') from 1 for 8),
    u.id,
    NOW(),
    NOW()
FROM users u
WHERE NOT EXISTS (
    SELECT 1
    FROM workspace_memberships wm
    WHERE wm.user_id = u.id
);

INSERT INTO workspaces (id, name, slug, created_by_user_id, created_at, updated_at)
SELECT gen_random_uuid(), 'Imported Workspace', 'imported-workspace', NULL, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM workspaces)
  AND (
      EXISTS (SELECT 1 FROM workflows)
      OR EXISTS (SELECT 1 FROM credentials)
      OR EXISTS (SELECT 1 FROM tags)
  );

INSERT INTO workspace_memberships (id, workspace_id, user_id, role, created_at, updated_at)
SELECT gen_random_uuid(), w.id, u.id, 'owner', NOW(), NOW()
FROM users u
INNER JOIN workspaces w ON w.created_by_user_id = u.id
LEFT JOIN workspace_memberships wm ON wm.workspace_id = w.id AND wm.user_id = u.id
WHERE wm.id IS NULL;

UPDATE users u
SET active_workspace_id = workspace_choice.workspace_id
FROM (
    SELECT DISTINCT ON (wm.user_id)
        wm.user_id,
        wm.workspace_id
    FROM workspace_memberships wm
    INNER JOIN workspaces w ON w.id = wm.workspace_id
    ORDER BY
        wm.user_id,
        CASE wm.role
            WHEN 'owner' THEN 0
            WHEN 'admin' THEN 1
            WHEN 'member' THEN 2
            ELSE 3
        END,
        w.created_at ASC
) AS workspace_choice
WHERE u.id = workspace_choice.user_id
  AND u.active_workspace_id IS NULL;

WITH default_workspace AS (
    SELECT id, created_by_user_id
    FROM workspaces
    ORDER BY created_at ASC
    LIMIT 1
)
UPDATE workflows
SET workspace_id = (SELECT id FROM default_workspace),
    owner_user_id = COALESCE(owner_user_id, (SELECT created_by_user_id FROM default_workspace))
WHERE workspace_id IS NULL;

WITH default_workspace AS (
    SELECT id, created_by_user_id
    FROM workspaces
    ORDER BY created_at ASC
    LIMIT 1
)
UPDATE credentials
SET workspace_id = (SELECT id FROM default_workspace),
    owner_user_id = COALESCE(owner_user_id, (SELECT created_by_user_id FROM default_workspace))
WHERE workspace_id IS NULL;

WITH default_workspace AS (
    SELECT id
    FROM workspaces
    ORDER BY created_at ASC
    LIMIT 1
)
UPDATE tags
SET workspace_id = (SELECT id FROM default_workspace)
WHERE workspace_id IS NULL;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'tags_name_key'
    ) THEN
        ALTER TABLE tags DROP CONSTRAINT tags_name_key;
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'users_active_workspace_id_fkey'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_active_workspace_id_fkey
            FOREIGN KEY (active_workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'workflows_workspace_id_fkey'
    ) THEN
        ALTER TABLE workflows
            ADD CONSTRAINT workflows_workspace_id_fkey
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'workflows_owner_user_id_fkey'
    ) THEN
        ALTER TABLE workflows
            ADD CONSTRAINT workflows_owner_user_id_fkey
            FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'credentials_workspace_id_fkey'
    ) THEN
        ALTER TABLE credentials
            ADD CONSTRAINT credentials_workspace_id_fkey
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'credentials_owner_user_id_fkey'
    ) THEN
        ALTER TABLE credentials
            ADD CONSTRAINT credentials_owner_user_id_fkey
            FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'tags_workspace_id_fkey'
    ) THEN
        ALTER TABLE tags
            ADD CONSTRAINT tags_workspace_id_fkey
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'tags_workspace_name_key'
    ) THEN
        ALTER TABLE tags
            ADD CONSTRAINT tags_workspace_name_key UNIQUE (workspace_id, name);
    END IF;
END $$;

ALTER TABLE workflows ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE credentials ALTER COLUMN workspace_id SET NOT NULL;
ALTER TABLE tags ALTER COLUMN workspace_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_active_workspace_id ON users(active_workspace_id);
CREATE INDEX IF NOT EXISTS idx_workflows_workspace_id ON workflows(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workflows_owner_user_id ON workflows(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_credentials_workspace_id ON credentials(workspace_id);
CREATE INDEX IF NOT EXISTS idx_credentials_owner_user_id ON credentials(owner_user_id);
CREATE INDEX IF NOT EXISTS idx_tags_workspace_id ON tags(workspace_id);
