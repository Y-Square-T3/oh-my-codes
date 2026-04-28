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
