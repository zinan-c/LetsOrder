CREATE TABLE menu_item_ratings (
    id TEXT PRIMARY KEY NOT NULL,
    menu_item_id TEXT NOT NULL,
    participant_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (menu_item_id) REFERENCES menu_items(id),
    FOREIGN KEY (participant_id) REFERENCES participants(id),
    UNIQUE(menu_item_id, participant_id)
);

CREATE INDEX idx_menu_item_ratings_menu_item_id ON menu_item_ratings(menu_item_id);
