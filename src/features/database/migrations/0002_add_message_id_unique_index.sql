DELETE FROM `token_usages`;

--> statement-breakpoint
CREATE UNIQUE INDEX `idx_token_usages_message_id` ON `token_usages` (`message_id`);
