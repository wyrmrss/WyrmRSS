-- Instance-wide settings move to local_instance.
ALTER TABLE local_instance ADD COLUMN feed_poll_interval_secs   INTEGER NOT NULL DEFAULT 900;
ALTER TABLE local_instance ADD COLUMN http_timeout_secs         INTEGER NOT NULL DEFAULT 30;
ALTER TABLE local_instance ADD COLUMN http_connect_timeout_secs INTEGER NOT NULL DEFAULT 10;
ALTER TABLE local_instance ADD COLUMN http_retries              INTEGER NOT NULL DEFAULT 3;
ALTER TABLE local_instance ADD COLUMN http_user_agent           TEXT;
ALTER TABLE local_instance ADD COLUMN rsshub_instance           TEXT;

UPDATE local_instance SET
    feed_poll_interval_secs   = s.feed_poll_interval_secs,
    http_timeout_secs         = s.http_timeout,
    http_connect_timeout_secs = s.http_connect_timeout,
    http_retries              = s.http_retries,
    http_user_agent           = s.http_user_agent,
    rsshub_instance           = s.rsshub_instance
FROM settings s;

-- SQLite can't change a primary key, so rebuild settings keyed by user_id.
-- No REFERENCES and DEFAULT 1, like the other user_id columns on the desktop.
CREATE TABLE settings_new (
    user_id                  INTEGER NOT NULL PRIMARY KEY DEFAULT 1,
    page_size                INTEGER NOT NULL DEFAULT 100,
    read_mode                TEXT NOT NULL DEFAULT 'on_open'
                             CHECK (read_mode IN ('on_open', 'on_scroll', 'manually', 'disabled')),
    expire_read_after_days   INTEGER,
    expire_unread_after_days INTEGER,
    read_delay_secs          INTEGER NOT NULL DEFAULT 0
);

-- Copies the old singleton row to every user.
INSERT INTO settings_new (user_id, page_size, read_mode, read_delay_secs,
                          expire_read_after_days, expire_unread_after_days)
SELECT u.id, s.page_size, s.read_mode, s.read_delay_secs,
       s.expire_read_after_days, s.expire_unread_after_days
FROM local_users u
CROSS JOIN settings s;

DROP TABLE settings;
ALTER TABLE settings_new RENAME TO settings;
