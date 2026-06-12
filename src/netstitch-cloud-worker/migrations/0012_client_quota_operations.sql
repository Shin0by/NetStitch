CREATE TABLE IF NOT EXISTS client_quota_operations (
  client_identifier TEXT NOT NULL,
  operation_kind TEXT NOT NULL,
  operation_id TEXT NOT NULL,
  window_ends_at_ms INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  PRIMARY KEY (client_identifier, operation_kind, operation_id)
);

CREATE INDEX IF NOT EXISTS idx_quota_operations_expiry
  ON client_quota_operations(window_ends_at_ms);
