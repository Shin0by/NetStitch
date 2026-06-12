PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS author_profiles (
  author_user_id TEXT PRIMARY KEY,
  author_signature TEXT NOT NULL,
  author_signature_norm TEXT NOT NULL UNIQUE,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  FOREIGN KEY (author_user_id) REFERENCES users(user_id)
);

CREATE INDEX IF NOT EXISTS idx_author_profiles_signature_norm
  ON author_profiles(author_signature_norm);
