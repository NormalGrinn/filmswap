CREATE TABLE users (
    discord_id INTEGER PRIMARY KEY,
    username TEXT UNIQUE,
    letter TEXT,
    submitted_gift TEXT,
    giftee_id INTEGER,
    is_banned INTEGER NOT NULL,
    has_joined INTEGER NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);