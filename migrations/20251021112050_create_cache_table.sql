-- Add migration script here
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS cache (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX idx_cache_expires_at ON cache(expires_at);