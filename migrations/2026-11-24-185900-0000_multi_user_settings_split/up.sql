-- Instance-wide settings move to local_instance.
ALTER TABLE local_instance
  ADD COLUMN feed_poll_interval_secs   INTEGER NOT NULL DEFAULT 900,
  ADD COLUMN http_timeout_secs         INTEGER NOT NULL DEFAULT 30,
  ADD COLUMN http_connect_timeout_secs INTEGER NOT NULL DEFAULT 10,
  ADD COLUMN http_retries              INTEGER NOT NULL DEFAULT 3,
  ADD COLUMN http_user_agent           TEXT,
  ADD COLUMN rsshub_instance           TEXT;

UPDATE local_instance SET
  feed_poll_interval_secs   = s.feed_poll_interval_secs,
  http_timeout_secs         = s.http_timeout,
  http_connect_timeout_secs = s.http_connect_timeout,
  http_retries              = s.http_retries,
  http_user_agent           = s.http_user_agent,
  rsshub_instance           = s.rsshub_instance
FROM settings s;

ALTER TABLE settings
  DROP COLUMN feed_poll_interval_secs,
  DROP COLUMN http_timeout,
  DROP COLUMN http_connect_timeout,
  DROP COLUMN http_retries,
  DROP COLUMN http_user_agent,
  DROP COLUMN rsshub_instance;

-- The remaining settings become per user, each user starts with the current values.
ALTER TABLE settings
  DROP CONSTRAINT settings_pkey,
  DROP CONSTRAINT only_one_row,
  ADD COLUMN user_id INT REFERENCES local_users(id) ON DELETE CASCADE;

-- Copies the old singleton row, the only one without a user_id, to every user.
INSERT INTO settings (user_id, page_size, read_mode, read_delay_secs,
                      expire_read_after_days, expire_unread_after_days)
SELECT u.id, s.page_size, s.read_mode, s.read_delay_secs,
       s.expire_read_after_days, s.expire_unread_after_days
FROM local_users u
CROSS JOIN settings s
WHERE s.user_id IS NULL;

DELETE FROM settings WHERE user_id IS NULL;

ALTER TABLE settings
  DROP COLUMN is_singleton,
  ALTER COLUMN user_id SET NOT NULL,
  ADD PRIMARY KEY (user_id);
