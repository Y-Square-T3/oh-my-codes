--> clear history records
DELETE FROM `token_usages`;

--> Add unique index on message_id
CREATE UNIQUE INDEX `token_usages_message_id_idx` ON `token_usages` (`message_id`);
