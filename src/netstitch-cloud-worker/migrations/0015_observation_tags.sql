CREATE TABLE IF NOT EXISTS user_tags (
  user_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  PRIMARY KEY (user_id, tag),
  FOREIGN KEY (user_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS observation_tags (
  visibility TEXT NOT NULL,
  app_id TEXT NOT NULL,
  owner_user_id TEXT NOT NULL DEFAULT '',
  ip TEXT NOT NULL,
  port INTEGER NOT NULL,
  protocol TEXT NOT NULL,
  tag_user_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL,
  PRIMARY KEY (
    visibility,
    app_id,
    owner_user_id,
    ip,
    port,
    protocol,
    tag_user_id,
    tag
  ),
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id),
  FOREIGN KEY (tag_user_id, tag) REFERENCES user_tags(user_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_user_tags_tag ON user_tags(tag);
CREATE INDEX IF NOT EXISTS idx_user_tags_expiry ON user_tags(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_observation_tags_lookup
  ON observation_tags(app_id, visibility, owner_user_id, tag, ip, port, protocol);
CREATE INDEX IF NOT EXISTS idx_observation_tags_expiry ON observation_tags(expires_at_ms);

CREATE TRIGGER IF NOT EXISTS observation_tags_visibility_insert_check
BEFORE INSERT ON observation_tags
WHEN NEW.visibility NOT IN ('public', 'private')
BEGIN
  SELECT RAISE(ABORT, 'invalid observation tag visibility');
END;
