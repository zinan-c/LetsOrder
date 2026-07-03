ALTER TABLE gatherings
ADD COLUMN is_locked INTEGER NOT NULL DEFAULT 0;

ALTER TABLE participants
ADD COLUMN last_menu_activity_at TEXT;

UPDATE gatherings
SET is_locked = 1
WHERE status = 'locked' OR locked_at IS NOT NULL;
