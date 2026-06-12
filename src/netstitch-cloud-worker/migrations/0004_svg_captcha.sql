CREATE TABLE IF NOT EXISTS captcha_challenges (
  challenge_id TEXT PRIMARY KEY,
  pool_key TEXT NOT NULL,
  purpose TEXT NOT NULL,
  scope_hash TEXT,
  salt TEXT NOT NULL,
  answer_hash TEXT NOT NULL,
  svg TEXT NOT NULL,
  issued_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  consumed_at_ms INTEGER
);

CREATE TABLE IF NOT EXISTS captcha_refresh_pools (
  pool_key TEXT PRIMARY KEY,
  purpose TEXT NOT NULL,
  scope_hash TEXT,
  challenge_ids_json TEXT NOT NULL,
  next_index INTEGER NOT NULL DEFAULT 0,
  created_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS captcha_failure_buckets (
  bucket_hash TEXT PRIMARY KEY,
  window_started_at_ms INTEGER NOT NULL,
  window_ends_at_ms INTEGER NOT NULL,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  consecutive_failures INTEGER NOT NULL DEFAULT 0,
  blocked_until_ms INTEGER NOT NULL DEFAULT 0,
  last_attempt_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_captcha_challenges_pool ON captcha_challenges(pool_key);
CREATE INDEX IF NOT EXISTS idx_captcha_challenges_expiry ON captcha_challenges(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_captcha_refresh_pools_expiry ON captcha_refresh_pools(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_captcha_failure_buckets_expiry ON captcha_failure_buckets(window_ends_at_ms);
