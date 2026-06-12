PRAGMA foreign_keys = ON;

UPDATE user_identities
SET display_name = NULL
WHERE display_name IS NOT NULL;
