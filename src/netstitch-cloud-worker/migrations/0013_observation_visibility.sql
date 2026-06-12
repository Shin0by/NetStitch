ALTER TABLE observations RENAME TO observations_previous_0013;

CREATE TABLE observations (
  app_id TEXT NOT NULL,
  visibility TEXT NOT NULL DEFAULT 'public',
  publisher_key TEXT NOT NULL DEFAULT '',
  ip TEXT NOT NULL,
  port INTEGER NOT NULL,
  protocol TEXT NOT NULL,
  connection_state TEXT NOT NULL,
  requests INTEGER NOT NULL,
  first_seen_ms INTEGER NOT NULL,
  last_seen_ms INTEGER NOT NULL,
  failed_hits INTEGER NOT NULL DEFAULT 0,
  successful_hits INTEGER NOT NULL DEFAULT 0,
  domain_raw TEXT,
  domain_verified TEXT,
  domain_status TEXT NOT NULL DEFAULT 'None',
  trust_level TEXT NOT NULL DEFAULT 'CommunityVerified',
  source_kind TEXT NOT NULL DEFAULT 'VerifiedUpload',
  app_signature_subject TEXT,
  app_signature_issuer TEXT,
  expires_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (visibility, app_id, publisher_key, ip, port, protocol),
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id)
);

INSERT INTO observations (
  app_id,
  visibility,
  publisher_key,
  ip,
  port,
  protocol,
  connection_state,
  requests,
  first_seen_ms,
  last_seen_ms,
  failed_hits,
  successful_hits,
  domain_raw,
  domain_verified,
  domain_status,
  trust_level,
  source_kind,
  app_signature_subject,
  app_signature_issuer,
  expires_at_ms,
  updated_at_ms
)
SELECT app_id,
       'public',
       publisher_key,
       ip,
       port,
       protocol,
       connection_state,
       requests,
       first_seen_ms,
       last_seen_ms,
       failed_hits,
       successful_hits,
       domain_raw,
       domain_verified,
       domain_status,
       trust_level,
       source_kind,
       app_signature_subject,
       app_signature_issuer,
       expires_at_ms,
       updated_at_ms
FROM observations_previous_0013;

DROP TABLE observations_previous_0013;

ALTER TABLE observation_author_rows RENAME TO observation_author_rows_previous_0013;

CREATE TABLE observation_author_rows (
  app_id TEXT NOT NULL,
  visibility TEXT NOT NULL DEFAULT 'public',
  author_user_id TEXT NOT NULL,
  publisher_key TEXT NOT NULL DEFAULT '',
  ip TEXT NOT NULL,
  port INTEGER NOT NULL,
  protocol TEXT NOT NULL,
  connection_state TEXT NOT NULL,
  requests INTEGER NOT NULL,
  first_seen_ms INTEGER NOT NULL,
  last_seen_ms INTEGER NOT NULL,
  failed_hits INTEGER NOT NULL DEFAULT 0,
  successful_hits INTEGER NOT NULL DEFAULT 0,
  domain_raw TEXT,
  domain_verified TEXT,
  domain_status TEXT NOT NULL DEFAULT 'None',
  trust_level TEXT NOT NULL DEFAULT 'CommunityVerified',
  source_kind TEXT NOT NULL DEFAULT 'VerifiedUpload',
  app_signature_subject TEXT,
  app_signature_issuer TEXT,
  expires_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (visibility, app_id, author_user_id, publisher_key, ip, port, protocol),
  FOREIGN KEY (app_id) REFERENCES app_catalog(app_id),
  FOREIGN KEY (author_user_id) REFERENCES users(user_id)
);

INSERT INTO observation_author_rows (
  app_id,
  visibility,
  author_user_id,
  publisher_key,
  ip,
  port,
  protocol,
  connection_state,
  requests,
  first_seen_ms,
  last_seen_ms,
  failed_hits,
  successful_hits,
  domain_raw,
  domain_verified,
  domain_status,
  trust_level,
  source_kind,
  app_signature_subject,
  app_signature_issuer,
  expires_at_ms,
  updated_at_ms
)
SELECT app_id,
       'public',
       author_user_id,
       publisher_key,
       ip,
       port,
       protocol,
       connection_state,
       requests,
       first_seen_ms,
       last_seen_ms,
       failed_hits,
       successful_hits,
       domain_raw,
       domain_verified,
       domain_status,
       trust_level,
       source_kind,
       app_signature_subject,
       app_signature_issuer,
       expires_at_ms,
       updated_at_ms
FROM observation_author_rows_previous_0013;

DROP TABLE observation_author_rows_previous_0013;

CREATE INDEX IF NOT EXISTS idx_observations_app ON observations(app_id);
CREATE INDEX IF NOT EXISTS idx_observations_expiry ON observations(expires_at_ms);
CREATE INDEX IF NOT EXISTS idx_observation_author_rows_author
  ON observation_author_rows(author_user_id, app_id);
CREATE INDEX IF NOT EXISTS idx_observation_author_rows_app
  ON observation_author_rows(app_id);
CREATE INDEX IF NOT EXISTS idx_observation_author_rows_expiry
  ON observation_author_rows(expires_at_ms);
