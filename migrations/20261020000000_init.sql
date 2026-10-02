-- IDs (uuid v7) and timestamps (RFC 3339 UTC) are generated in Rust, not by
-- SQL defaults, so this schema ports to Postgres with type changes only.

CREATE TABLE tickets (
    id                TEXT PRIMARY KEY NOT NULL,
    number            INTEGER NOT NULL UNIQUE,
    subject           TEXT NOT NULL,
    body              TEXT NOT NULL,
    severity          TEXT NOT NULL,
    status            TEXT NOT NULL,
    customer_name     TEXT,
    customer_email    TEXT,
    anon_animal       TEXT,
    anon_number       INTEGER,
    access_token_hash TEXT NOT NULL UNIQUE,
    assignee_name     TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    closed_at         TEXT
);

CREATE INDEX tickets_status_number ON tickets (status, number);

CREATE TABLE messages (
    id           TEXT PRIMARY KEY NOT NULL,
    ticket_id    TEXT NOT NULL REFERENCES tickets (id) ON DELETE CASCADE,
    author_kind  TEXT NOT NULL,
    author_name  TEXT,
    body         TEXT NOT NULL,
    internal     BOOLEAN NOT NULL DEFAULT FALSE,
    system_event TEXT, -- JSON, see nullbox_domain::SystemEvent
    created_at   TEXT NOT NULL
);

CREATE INDEX messages_ticket_created ON messages (ticket_id, created_at);
