-- Rate limits the manual refresh button per user.
ALTER TABLE local_users ADD COLUMN last_manual_poll_at TIMESTAMP;

-- 0 = no limit, since the desktop app has a single user.
ALTER TABLE local_instance ADD COLUMN manual_poll_interval_secs INTEGER NOT NULL DEFAULT 0;
