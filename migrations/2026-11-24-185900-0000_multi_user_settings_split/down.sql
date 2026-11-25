-- Keeps one row for the singleton table, the first admin's settings.
DELETE FROM settings WHERE user_id <> (
  SELECT s.user_id
  FROM settings s
  JOIN local_users u ON u.id = s.user_id
  ORDER BY u.is_admin DESC, u.id
  LIMIT 1
);

ALTER TABLE settings
  DROP CONSTRAINT settings_pkey,
  DROP COLUMN user_id,
  ADD COLUMN is_singleton BOOL NOT NULL DEFAULT TRUE,
  ADD PRIMARY KEY (is_singleton),
  ADD CONSTRAINT only_one_row CHECK (is_singleton = TRUE),
  ADD COLUMN feed_poll_interval_secs INTEGER NOT NULL DEFAULT 900,
  ADD COLUMN http_timeout            INTEGER NOT NULL DEFAULT 30,
  ADD COLUMN http_connect_timeout    INTEGER NOT NULL DEFAULT 10,
  ADD COLUMN http_retries            INTEGER NOT NULL DEFAULT 3,
  ADD COLUMN http_user_agent         TEXT,
  ADD COLUMN rsshub_instance         TEXT DEFAULT NULL;

-- Without users, up.sql left no settings rows.
INSERT INTO settings (is_singleton)
SELECT TRUE
WHERE NOT EXISTS (SELECT 1 FROM settings);

UPDATE settings SET
  feed_poll_interval_secs = i.feed_poll_interval_secs,
  http_timeout            = i.http_timeout_secs,
  http_connect_timeout    = i.http_connect_timeout_secs,
  http_retries            = i.http_retries,
  http_user_agent         = i.http_user_agent,
  rsshub_instance         = i.rsshub_instance
FROM local_instance i;

ALTER TABLE local_instance
  DROP COLUMN feed_poll_interval_secs,
  DROP COLUMN http_timeout_secs,
  DROP COLUMN http_connect_timeout_secs,
  DROP COLUMN http_retries,
  DROP COLUMN http_user_agent,
  DROP COLUMN rsshub_instance;
