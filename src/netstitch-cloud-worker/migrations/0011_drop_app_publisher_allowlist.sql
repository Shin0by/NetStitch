-- Cloud upload no longer uses a server-side preapproved publisher table.
-- Application catalog rows are created or refreshed from authenticated upload payloads.
DROP TABLE IF EXISTS app_publisher_allowlist;
