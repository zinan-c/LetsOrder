CREATE UNIQUE INDEX IF NOT EXISTS idx_participants_gathering_user
ON participants (gathering_id, user_id)
WHERE user_id IS NOT NULL;
