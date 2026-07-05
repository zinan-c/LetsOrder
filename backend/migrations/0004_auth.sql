CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'user')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auth_sessions (
    token TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

ALTER TABLE participants
ADD COLUMN user_id TEXT REFERENCES users(id);

CREATE INDEX idx_auth_sessions_user_id ON auth_sessions(user_id);
CREATE INDEX idx_participants_user_id ON participants(user_id);

INSERT INTO users (
    id, username, display_name, password_hash, role, created_at, updated_at
)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'suite-admin',
    'suite-admin',
    'acf17b5dbb8c77b6',
    'admin',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);
