--> clear history records
DELETE FROM `token_usages`;

--> Add unique index on message_id
CREATE UNIQUE INDEX `idx_token_usages_message_id` ON `token_usages` (`message_id`);
