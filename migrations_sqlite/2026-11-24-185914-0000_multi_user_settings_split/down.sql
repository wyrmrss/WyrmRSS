-- Rebuilds the singleton settings table from the first admin's row and local_instance.
CREATE TABLE settings_old (
    is_singleton             BOOLEAN NOT NULL PRIMARY KEY DEFAULT TRUE,
    page_size                INTEGER NOT NULL DEFAULT 100,
    feed_poll_interval_secs  INTEGER NOT NULL DEFAULT 900,
    http_timeout             INTEGER NOT NULL DEFAULT 30,
    http_connect_timeout     INTEGER NOT NULL DEFAULT 10,
    http_retries             INTEGER NOT NULL DEFAULT 3,
    http_user_agent          TEXT,
    read_mode                TEXT NOT NULL DEFAULT 'on_open'
                             CHECK (read_mode IN ('on_open', 'on_scroll', 'manually', 'disabled')),
    expire_read_after_days   INTEGER,
    expire_unread_after_days INTEGER,
    read_delay_secs          INTEGER NOT NULL DEFAULT 0,
    rsshub_instance          TEXT DEFAULT NULL,
    CONSTRAINT only_one_row CHECK (is_singleton = TRUE)
);

INSERT INTO settings_old (
    is_singleton, page_size, feed_poll_interval_secs, http_timeout,
    http_connect_timeout, http_retries, http_user_agent, read_mode,
    expire_read_after_days, expire_unread_after_days, read_delay_secs, rsshub_instance
)
SELECT
    TRUE, s.page_size, i.feed_poll_interval_secs, i.http_timeout_secs,
    i.http_connect_timeout_secs, i.http_retries, i.http_user_agent, s.read_mode,
    s.expire_read_after_days, s.expire_unread_after_days, s.read_delay_secs, i.rsshub_instance
FROM settings s
JOIN local_users u ON u.id = s.user_id
CROSS JOIN local_instance i
ORDER BY u.is_admin DESC, u.id
LIMIT 1;

DROP TABLE settings;
ALTER TABLE settings_old RENAME TO settings;

ALTER TABLE local_instance DROP COLUMN feed_poll_interval_secs;
ALTER TABLE local_instance DROP COLUMN http_timeout_secs;
ALTER TABLE local_instance DROP COLUMN http_connect_timeout_secs;
ALTER TABLE local_instance DROP COLUMN http_retries;
ALTER TABLE local_instance DROP COLUMN http_user_agent;
ALTER TABLE local_instance DROP COLUMN rsshub_instance;
