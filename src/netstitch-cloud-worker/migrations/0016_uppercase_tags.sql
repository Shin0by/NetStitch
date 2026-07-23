PRAGMA defer_foreign_keys = ON;

DELETE FROM observation_tags
WHERE length(tag) NOT BETWEEN 1 AND 16
   OR tag GLOB '*[^A-Za-z0-9._-]*'
   OR tag NOT GLOB '*[A-Za-z0-9]*';

DELETE FROM user_tags
WHERE length(tag) NOT BETWEEN 1 AND 16
   OR tag GLOB '*[^A-Za-z0-9._-]*'
   OR tag NOT GLOB '*[A-Za-z0-9]*';

UPDATE user_tags
SET tag = UPPER(tag)
WHERE tag != UPPER(tag);

UPDATE observation_tags
SET tag = UPPER(tag)
WHERE tag != UPPER(tag);

CREATE TRIGGER IF NOT EXISTS user_tags_format_insert_check
BEFORE INSERT ON user_tags
WHEN NEW.tag != UPPER(NEW.tag)
  OR length(NEW.tag) NOT BETWEEN 1 AND 16
  OR NEW.tag GLOB '*[^A-Z0-9._-]*'
  OR NEW.tag NOT GLOB '*[A-Z0-9]*'
BEGIN
  SELECT RAISE(ABORT, 'invalid tag format');
END;

CREATE TRIGGER IF NOT EXISTS user_tags_format_update_check
BEFORE UPDATE OF tag ON user_tags
WHEN NEW.tag != UPPER(NEW.tag)
  OR length(NEW.tag) NOT BETWEEN 1 AND 16
  OR NEW.tag GLOB '*[^A-Z0-9._-]*'
  OR NEW.tag NOT GLOB '*[A-Z0-9]*'
BEGIN
  SELECT RAISE(ABORT, 'invalid tag format');
END;

CREATE TRIGGER IF NOT EXISTS observation_tags_format_insert_check
BEFORE INSERT ON observation_tags
WHEN NEW.tag != UPPER(NEW.tag)
  OR length(NEW.tag) NOT BETWEEN 1 AND 16
  OR NEW.tag GLOB '*[^A-Z0-9._-]*'
  OR NEW.tag NOT GLOB '*[A-Z0-9]*'
BEGIN
  SELECT RAISE(ABORT, 'invalid observation tag format');
END;

CREATE TRIGGER IF NOT EXISTS observation_tags_format_update_check
BEFORE UPDATE OF tag ON observation_tags
WHEN NEW.tag != UPPER(NEW.tag)
  OR length(NEW.tag) NOT BETWEEN 1 AND 16
  OR NEW.tag GLOB '*[^A-Z0-9._-]*'
  OR NEW.tag NOT GLOB '*[A-Z0-9]*'
BEGIN
  SELECT RAISE(ABORT, 'invalid observation tag format');
END;
