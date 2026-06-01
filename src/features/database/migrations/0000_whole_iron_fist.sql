CREATE TABLE `account_state` (
	`id` integer PRIMARY KEY NOT NULL,
	`active_account_id` text,
	`active_workspace_id` text
);
--> statement-breakpoint
CREATE TABLE `accounts` (
	`id` text PRIMARY KEY NOT NULL,
	`email` text NOT NULL,
	`url` text NOT NULL,
	`access_token` text NOT NULL,
	`refresh_token` text NOT NULL,
	`token_expiry` integer
);
--> statement-breakpoint
CREATE TABLE `models` (
	`id` text NOT NULL,
	`provider_id` text NOT NULL,
	`name` text NOT NULL,
	`family` text,
	`attachment` integer,
	`reasoning` integer,
	`tool_call` integer,
	`enable` integer,
	`structured_output` integer,
	`temperature` integer,
	`interleaved_field` text,
	`knowledge` text,
	`release_date` text,
	`last_updated` text,
	`open_weights` integer,
	`modalities_input` text,
	`modalities_output` text,
	`cost_input` real DEFAULT 0,
	`cost_output` real DEFAULT 0,
	`cost_reasoning` real DEFAULT 0,
	`cost_cache_read` real DEFAULT 0,
	`cost_cache_write` real DEFAULT 0,
	`limit_context` integer,
	`limit_output` integer,
	`account_id` text NOT NULL,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL,
	PRIMARY KEY (`id`, `provider_id`, `account_id`)
);
--> statement-breakpoint
CREATE TABLE `providers` (
	`id` text NOT NULL,
	`name` text NOT NULL,
	`api` text,
	`npm` text,
	`doc` text,
	`env_vars` text NOT NULL,
	`account_id` text NOT NULL,
	`last_fetched_at` integer,
	`created_at` integer NOT NULL,
	`updated_at` integer NOT NULL,
	PRIMARY KEY (`id`, `account_id`)
);
