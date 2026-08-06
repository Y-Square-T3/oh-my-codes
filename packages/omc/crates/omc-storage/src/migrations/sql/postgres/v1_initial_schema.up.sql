CREATE TABLE IF NOT EXISTS channel (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    topic TEXT,
    created_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS message (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    edited_at BIGINT,
    reply_to TEXT
);

CREATE INDEX IF NOT EXISTS idx_message_channel_ts ON message(channel_id, timestamp);

CREATE TABLE IF NOT EXISTS account (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    url TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry BIGINT NOT NULL,
    active_workspace_id TEXT
);

CREATE TABLE IF NOT EXISTS workspace (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id)
);

CREATE INDEX IF NOT EXISTS idx_workspace_account ON workspace(account_id);

CREATE TABLE IF NOT EXISTS active_account (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    account_id TEXT,
    FOREIGN KEY (account_id) REFERENCES account(id)
);

CREATE TABLE IF NOT EXISTS provider (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    name TEXT NOT NULL,
    env TEXT NOT NULL,
    api TEXT,
    npm TEXT,
    doc TEXT,
    models TEXT NOT NULL,
    account_id TEXT NOT NULL,
    last_fetched_at BIGINT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id)
);

CREATE INDEX IF NOT EXISTS idx_provider_account ON provider(account_id);

CREATE TABLE IF NOT EXISTS token_usage (
    id TEXT PRIMARY KEY,
    client TEXT NOT NULL,
    session_id TEXT NOT NULL,
    message_id TEXT NOT NULL UNIQUE,
    agent TEXT,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    input_tokens BIGINT NOT NULL,
    output_tokens BIGINT NOT NULL,
    reasoning_tokens BIGINT NOT NULL,
    cache_read_tokens BIGINT NOT NULL,
    cache_write_tokens BIGINT NOT NULL,
    pushed BOOLEAN NOT NULL DEFAULT FALSE,
    recorded_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_token_usage_pushed ON token_usage(pushed, recorded_at);
