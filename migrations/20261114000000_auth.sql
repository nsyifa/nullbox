
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT,
    display_name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL
        REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX sessions_user_id_idx
    ON sessions(user_id);

CREATE INDEX sessions_expires_at_idx
    ON sessions(expires_at);

CREATE TABLE oauth_identities (
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    user_id TEXT NOT NULL
        REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (provider, provider_user_id)
);
