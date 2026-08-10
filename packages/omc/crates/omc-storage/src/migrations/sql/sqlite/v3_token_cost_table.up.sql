CREATE TABLE token_cost (
    usage_id TEXT PRIMARY KEY NOT NULL REFERENCES token_usage(id),
    input_cost_micros INTEGER NOT NULL DEFAULT 0,
    output_cost_micros INTEGER NOT NULL DEFAULT 0,
    reasoning_cost_micros INTEGER NOT NULL DEFAULT 0,
    cache_read_cost_micros INTEGER NOT NULL DEFAULT 0,
    cache_write_cost_micros INTEGER NOT NULL DEFAULT 0,
    audio_input_cost_micros INTEGER NOT NULL DEFAULT 0,
    video_input_cost_micros INTEGER NOT NULL DEFAULT 0,
    image_input_cost_micros INTEGER NOT NULL DEFAULT 0,
    total_cost_micros INTEGER NOT NULL DEFAULT 0
);
