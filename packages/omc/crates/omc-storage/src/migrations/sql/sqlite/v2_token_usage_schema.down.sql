CREATE TABLE token_usage_old (
    id TEXT PRIMARY KEY,
    client TEXT NOT NULL,
    session_id TEXT NOT NULL,
    message_id TEXT NOT NULL UNIQUE,
    agent TEXT,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    reasoning_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL,
    cache_write_tokens INTEGER NOT NULL,
    pushed INTEGER NOT NULL DEFAULT 0,
    recorded_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

INSERT INTO token_usage_old (id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at)
SELECT
    id,
    substr(agent, 1, instr(agent, '/') - 1),
    session_id,
    json_extract(metadata, '$.messageId'),
    substr(agent, instr(agent, '/') + 1),
    substr(model, 1, instr(model, '/') - 1),
    substr(model, instr(model, '/') + 1),
    input_tokens,
    output_tokens,
    reasoning_tokens,
    cache_read_tokens,
    cache_write_tokens,
    pushed,
    recorded_at,
    created_at
FROM token_usage;

DROP TABLE token_usage;

ALTER TABLE token_usage_old RENAME TO token_usage;

CREATE INDEX IF NOT EXISTS idx_token_usage_pushed ON token_usage(pushed, recorded_at);
