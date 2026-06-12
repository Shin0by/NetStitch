CREATE TABLE IF NOT EXISTS browser_verification_challenges (
  challenge_id TEXT PRIMARY KEY,
  purpose TEXT NOT NULL,
  scope_hash TEXT NOT NULL,
  nonce TEXT NOT NULL,
  memory_seed TEXT NOT NULL,
  difficulty INTEGER NOT NULL,
  memory_size_kib INTEGER NOT NULL,
  memory_rounds INTEGER NOT NULL,
  issued_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  consumed_at_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_browser_verification_scope
  ON browser_verification_challenges(purpose, scope_hash);

CREATE INDEX IF NOT EXISTS idx_browser_verification_expiry
  ON browser_verification_challenges(expires_at_ms);
