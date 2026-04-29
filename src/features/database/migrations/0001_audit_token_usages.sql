CREATE TABLE `token_usages` (
  `id` text PRIMARY KEY NOT NULL,
  `recorded_at` integer NOT NULL,
  `session_id` text NOT NULL,
  `message_id` text NOT NULL,
  `agent` text,
  `provider_id` text NOT NULL,
  `model_id` text NOT NULL,
  `input_tokens` integer NOT NULL,
  `output_tokens` integer NOT NULL,
  `reasoning_tokens` integer NOT NULL DEFAULT 0,
  `cache_read_tokens` integer NOT NULL DEFAULT 0,
  `cache_write_tokens` integer NOT NULL DEFAULT 0,
  `pushed` integer NOT NULL DEFAULT 0,
  `created_at` integer NOT NULL
);
CREATE INDEX `idx_token_usages_pushed` ON `token_usages` (`pushed`);
CREATE INDEX `idx_token_usages_recorded_at` ON `token_usages` (`recorded_at`);