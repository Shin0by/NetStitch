PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
  user_id TEXT PRIMARY KEY,
  login TEXT NOT NULL UNIQUE,
  password_salt TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  password_params_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  last_login_at_ms INTEGER
);

CREATE TABLE IF NOT EXISTS clients (
  client_id TEXT PRIMARY KEY,
  user_id TEXT,
  client_identifier TEXT NOT NULL UNIQUE,
  created_at_ms INTEGER NOT NULL,
  last_seen_at_ms INTEGER,
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS client_keys (
  key_id TEXT PRIMARY KEY,
  client_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  public_key_jwk TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active', 'retired', 'blocked')),
  created_at_ms INTEGER NOT NULL,
  retired_at_ms INTEGER,
  FOREIGN KEY (client_id) REFERENCES clients(client_id),
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS user_sessions (
  session_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  client_id TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  created_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  revoked_at_ms INTEGER,
  FOREIGN KEY (user_id) REFERENCES users(user_id),
  FOREIGN KEY (client_id) REFERENCES clients(client_id)
);

CREATE TABLE IF NOT EXISTS app_catalog (
  app_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  publisher_name TEXT,
  short_app_id TEXT,
  status TEXT NOT NULL CHECK (status IN ('active', 'retired', 'blocked')),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS observations (
  app_id TEXT NOT NULL,
  ip TEXT NOT NULL,
  port INTEGER NOT NULL,
  protocol TEXT NOT NULL,
  connection_state TEXT NOT NULL,
  requests INTEGER NOT NULL DEFAULT 0,
  first_seen_ms INTEGER NOT NULL,
  last_seen_ms INTEGER NOT NULL,
  failed_hits INTEGER NOT NULL DEFAULT 0,
  successful_hits INTEGER NOT NULL DEFAULT 0,
  domain_raw TEXT,
  domain_verified TEXT,
  domain_status TEXT NOT NULL,
  trust_level TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (app_id, ip, port, protocol),
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id)
);

CREATE TABLE IF NOT EXISTS observation_author_rows (
  app_id TEXT NOT NULL,
  author_user_id TEXT NOT NULL,
  ip TEXT NOT NULL,
  port INTEGER NOT NULL,
  protocol TEXT NOT NULL,
  connection_state TEXT NOT NULL,
  requests INTEGER NOT NULL DEFAULT 0,
  first_seen_ms INTEGER NOT NULL,
  last_seen_ms INTEGER NOT NULL,
  failed_hits INTEGER NOT NULL DEFAULT 0,
  successful_hits INTEGER NOT NULL DEFAULT 0,
  domain_raw TEXT,
  domain_verified TEXT,
  domain_status TEXT NOT NULL,
  trust_level TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (app_id, author_user_id, ip, port, protocol),
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id),
  FOREIGN KEY (author_user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS observation_submissions (
  submission_id TEXT PRIMARY KEY,
  app_id TEXT NOT NULL,
  author_user_id TEXT NOT NULL,
  client_id TEXT NOT NULL,
  row_count INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id),
  FOREIGN KEY (author_user_id) REFERENCES users(user_id),
  FOREIGN KEY (client_id) REFERENCES clients(client_id)
);

CREATE TABLE IF NOT EXISTS client_quota_windows (
  client_identifier TEXT NOT NULL,
  operation_kind TEXT NOT NULL,
  window_started_at_ms INTEGER NOT NULL,
  window_ends_at_ms INTEGER NOT NULL,
  limit_count INTEGER NOT NULL,
  used_count INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (client_identifier, operation_kind)
);

CREATE TABLE IF NOT EXISTS jwt_replay_cache (
  jti TEXT PRIMARY KEY,
  client_id TEXT NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS domain_verifications (
  app_id TEXT NOT NULL,
  ip TEXT NOT NULL,
  domain TEXT NOT NULL,
  status TEXT NOT NULL,
  checked_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  PRIMARY KEY (app_id, ip, domain)
);

CREATE TABLE IF NOT EXISTS audit_events (
  event_id TEXT PRIMARY KEY,
  created_at_ms INTEGER NOT NULL,
  entity_type TEXT NOT NULL,
  action_type TEXT NOT NULL,
  entity_id TEXT,
  actor_id TEXT,
  payload_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_login ON users(login);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON user_sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON user_sessions(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_app_catalog_display_name ON app_catalog(display_name);
CREATE INDEX IF NOT EXISTS idx_observations_app ON observations(app_id);
CREATE INDEX IF NOT EXISTS idx_observations_expiry ON observations(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_author_rows_app_author ON observation_author_rows(app_id, author_user_id);
CREATE INDEX IF NOT EXISTS idx_author_rows_expiry ON observation_author_rows(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_submissions_author ON observation_submissions(author_user_id, created_at_ms);
CREATE INDEX IF NOT EXISTS idx_quota_expiry ON client_quota_windows(window_ends_at_ms);
CREATE INDEX IF NOT EXISTS idx_jwt_replay_expiry ON jwt_replay_cache(expires_at_ms);
