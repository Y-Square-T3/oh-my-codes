CREATE TABLE token_usage_new (
    id TEXT PRIMARY KEY,
    workspace_id TEXT,
    session_id TEXT NOT NULL,
    agent TEXT NOT NULL,
    model TEXT NOT NULL,
    metadata TEXT,
    input_tokens BIGINT NOT NULL,
    output_tokens BIGINT NOT NULL,
    reasoning_tokens BIGINT NOT NULL,
    cache_read_tokens BIGINT NOT NULL,
    cache_write_tokens BIGINT NOT NULL,
    audio_input_tokens BIGINT NOT NULL DEFAULT 0,
    video_input_tokens BIGINT NOT NULL DEFAULT 0,
    image_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    pushed BOOLEAN NOT NULL DEFAULT FALSE,
    recorded_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

INSERT INTO token_usage_new (id, workspace_id, session_id, agent, model, metadata, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, audio_input_tokens, video_input_tokens, image_input_tokens, total_tokens, pushed, recorded_at, created_at)
SELECT
    id,
    NULL,
    session_id,
    client || '/' || COALESCE(agent, 'unknown'),
    provider_id || '/' || model_id,
    jsonb_build_object('messageId', message_id)::text,
    input_tokens,
    output_tokens,
    reasoning_tokens,
    cache_read_tokens,
    cache_write_tokens,
    0,
    0,
    0,
    input_tokens + output_tokens + reasoning_tokens + cache_read_tokens + cache_write_tokens,
    pushed,
    recorded_at,
    created_at
FROM token_usage;

DROP TABLE token_usage;

ALTER TABLE token_usage_new RENAME TO token_usage;

CREATE INDEX idx_token_usage_pushed ON token_usage(pushed, recorded_at);
CREATE INDEX idx_token_usage_workspace_session ON token_usage(workspace_id, session_id);
