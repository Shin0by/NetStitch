PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user_identities (
  provider TEXT NOT NULL,
  provider_subject_hash TEXT NOT NULL,
  user_id TEXT NOT NULL,
  email_hash TEXT,
  email_verified INTEGER NOT NULL DEFAULT 0,
  display_name TEXT,
  created_at_ms INTEGER NOT NULL,
  last_seen_at_ms INTEGER,
  PRIMARY KEY (provider, provider_subject_hash),
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS oauth_states (
  state TEXT PRIMARY KEY,
  poll_secret_hash TEXT NOT NULL,
  provider TEXT NOT NULL,
  client_identifier TEXT NOT NULL,
  client_public_key_jwk TEXT NOT NULL,
  pkce_verifier TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('created', 'captcha_ok', 'completed', 'failed', 'consumed')),
  result_json TEXT,
  failure_code TEXT,
  created_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  used_at_ms INTEGER
);

CREATE TABLE IF NOT EXISTS auth_attempt_windows (
  bucket_hash TEXT PRIMARY KEY,
  window_started_at_ms INTEGER NOT NULL,
  window_ends_at_ms INTEGER NOT NULL,
  limit_count INTEGER NOT NULL,
  used_count INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_identities_user ON user_identities(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_states_expiry ON oauth_states(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_auth_attempt_windows_expiry ON auth_attempt_windows(window_ends_at_ms);
