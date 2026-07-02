CREATE TABLE gatherings (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    invite_code TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('draft', 'active', 'locked', 'archived')),
    starts_at TEXT,
    expires_at TEXT NOT NULL,
    locked_at TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE participants (
    id TEXT PRIMARY KEY NOT NULL,
    gathering_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('host', 'participant')),
    access_token_hash TEXT NOT NULL,
    joined_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (gathering_id) REFERENCES gatherings(id)
);

CREATE TABLE menu_items (
    id TEXT PRIMARY KEY NOT NULL,
    gathering_id TEXT NOT NULL,
    created_by TEXT NOT NULL,
    updated_by TEXT,
    name TEXT NOT NULL,
    category TEXT,
    quantity INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit TEXT,
    owner_name TEXT,
    note TEXT,
    status TEXT NOT NULL CHECK (status IN ('planned', 'prepared', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (gathering_id) REFERENCES gatherings(id),
    FOREIGN KEY (created_by) REFERENCES participants(id),
    FOREIGN KEY (updated_by) REFERENCES participants(id)
);

CREATE TABLE photos (
    id TEXT PRIMARY KEY NOT NULL,
    gathering_id TEXT NOT NULL,
    uploaded_by TEXT NOT NULL,
    file_url TEXT NOT NULL,
    thumbnail_url TEXT,
    caption TEXT,
    taken_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (gathering_id) REFERENCES gatherings(id),
    FOREIGN KEY (uploaded_by) REFERENCES participants(id)
);

CREATE TABLE activity_logs (
    id TEXT PRIMARY KEY NOT NULL,
    gathering_id TEXT NOT NULL,
    actor_id TEXT,
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT,
    detail TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (gathering_id) REFERENCES gatherings(id),
    FOREIGN KEY (actor_id) REFERENCES participants(id)
);

CREATE INDEX idx_participants_gathering_id ON participants(gathering_id);
CREATE INDEX idx_menu_items_gathering_id ON menu_items(gathering_id);
CREATE INDEX idx_photos_gathering_id ON photos(gathering_id);
CREATE INDEX idx_activity_logs_gathering_id ON activity_logs(gathering_id);
