-- agent-runtime server persistence (PROJ-88). Idempotent on startup.

CREATE TABLE IF NOT EXISTS dispatch_tasks (
    id UUID PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    runtime_id TEXT NOT NULL,
    issue_key TEXT NOT NULL,
    pipeline TEXT NOT NULL,
    phase TEXT NOT NULL,
    run_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dispatch_tasks_runtime_phase
    ON dispatch_tasks (runtime_id, phase, created_at);

CREATE TABLE IF NOT EXISTS confirm_tasks (
    id UUID PRIMARY KEY,
    runtime_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    stage TEXT NOT NULL,
    phase TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_confirm_tasks_runtime_phase
    ON confirm_tasks (runtime_id, phase, created_at);

CREATE TABLE IF NOT EXISTS runtime_heartbeats (
    runtime_id TEXT PRIMARY KEY,
    last_heartbeat_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS run_mirrors (
    run_id TEXT PRIMARY KEY,
    issue_key TEXT,
    pipeline TEXT NOT NULL,
    run_status TEXT NOT NULL,
    current_stage TEXT NOT NULL,
    stages JSONB NOT NULL DEFAULT '[]',
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_run_mirrors_updated_at ON run_mirrors (updated_at DESC);

CREATE TABLE IF NOT EXISTS run_logs (
    id BIGSERIAL PRIMARY KEY,
    run_id TEXT NOT NULL,
    ts BIGINT NOT NULL,
    level TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_run_logs_run_id_ts ON run_logs (run_id, ts);

-- P9 Intake Chat (PDR-002), persistence wiring deferred — tables reserved for migration.
CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    runtime_id TEXT NOT NULL,
    product_id TEXT,
    status TEXT NOT NULL,
    draft_title TEXT,
    draft_pipeline TEXT,
    draft_description TEXT,
    linked_issue_key TEXT,
    linked_run_id TEXT,
    updated_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES chat_sessions(id),
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    ts BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_session ON chat_messages (session_id, ts);

CREATE TABLE IF NOT EXISTS chat_turn_tasks (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    runtime_id TEXT NOT NULL,
    user_message_id UUID NOT NULL,
    user_content TEXT NOT NULL,
    phase TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_turn_tasks_runtime ON chat_turn_tasks (runtime_id, phase, created_at);

CREATE TABLE IF NOT EXISTS bootstrap_tasks (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    runtime_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    draft_title TEXT NOT NULL,
    draft_pipeline TEXT NOT NULL,
    draft_description TEXT NOT NULL,
    phase TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bootstrap_tasks_runtime ON bootstrap_tasks (runtime_id, phase, created_at);
