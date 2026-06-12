PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS oauth_start_windows (
  bucket_hash TEXT PRIMARY KEY,
  window_started_at_ms INTEGER NOT NULL,
  window_ends_at_ms INTEGER NOT NULL,
  limit_count INTEGER NOT NULL,
  used_count INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_oauth_start_windows_expiry ON oauth_start_windows(window_ends_at_ms);
