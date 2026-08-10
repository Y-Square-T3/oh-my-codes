CREATE TABLE token_usage_old (
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

INSERT INTO token_usage_old (id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at)
SELECT
    id,
    substr(agent, 1, strpos(agent, '/') - 1),
    session_id,
    metadata::json->>'messageId',
    substr(agent, strpos(agent, '/') + 1),
    substr(model, 1, strpos(model, '/') - 1),
    substr(model, strpos(model, '/') + 1),
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

CREATE INDEX idx_token_usage_pushed ON token_usage(pushed, recorded_at);
