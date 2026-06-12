UPDATE observations
SET visibility = LOWER(visibility)
WHERE visibility IS NOT NULL
  AND visibility != LOWER(visibility);

UPDATE observation_author_rows
SET visibility = LOWER(visibility)
WHERE visibility IS NOT NULL
  AND visibility != LOWER(visibility);

CREATE TRIGGER IF NOT EXISTS observations_visibility_insert_check
BEFORE INSERT ON observations
WHEN NEW.visibility NOT IN ('public', 'private')
BEGIN
  SELECT RAISE(ABORT, 'invalid observation visibility');
END;

CREATE TRIGGER IF NOT EXISTS observations_visibility_update_check
BEFORE UPDATE OF visibility ON observations
WHEN NEW.visibility NOT IN ('public', 'private')
BEGIN
  SELECT RAISE(ABORT, 'invalid observation visibility');
END;

CREATE TRIGGER IF NOT EXISTS observation_author_rows_visibility_insert_check
BEFORE INSERT ON observation_author_rows
WHEN NEW.visibility NOT IN ('public', 'private')
BEGIN
  SELECT RAISE(ABORT, 'invalid observation visibility');
END;

CREATE TRIGGER IF NOT EXISTS observation_author_rows_visibility_update_check
BEFORE UPDATE OF visibility ON observation_author_rows
WHEN NEW.visibility NOT IN ('public', 'private')
BEGIN
  SELECT RAISE(ABORT, 'invalid observation visibility');
END;
