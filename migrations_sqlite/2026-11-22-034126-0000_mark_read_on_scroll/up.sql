-- SQLite can't alter a CHECK constraint, so rebuild the table with 'on_scroll' allowed.
CREATE TABLE settings_new (
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

INSERT INTO settings_new (
    is_singleton, page_size, feed_poll_interval_secs, http_timeout,
    http_connect_timeout, http_retries, http_user_agent, read_mode,
    expire_read_after_days, expire_unread_after_days, read_delay_secs, rsshub_instance
)
SELECT
    is_singleton, page_size, feed_poll_interval_secs, http_timeout,
    http_connect_timeout, http_retries, http_user_agent, read_mode,
    expire_read_after_days, expire_unread_after_days, read_delay_secs, rsshub_instance
FROM settings;

DROP TABLE settings;
ALTER TABLE settings_new RENAME TO settings;
