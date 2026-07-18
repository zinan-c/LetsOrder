CREATE TABLE IF NOT EXISTS websocket_tickets (
    ticket TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_websocket_tickets_expires_at
ON websocket_tickets(expires_at);
