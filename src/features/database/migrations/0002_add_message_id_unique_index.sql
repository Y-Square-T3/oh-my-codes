-- Deduplicate existing records by messageID, keeping the most recent one
DELETE FROM `token_usages`
WHERE `id` NOT IN (
  SELECT `id` FROM (
    SELECT `id`, MAX(`recorded_at`) as max_ra
    FROM `token_usages`
    GROUP BY `message_id`
  )
);

-- Add unique index on message_id
CREATE UNIQUE INDEX `token_usages_message_id_idx` ON `token_usages` (`message_id`);
