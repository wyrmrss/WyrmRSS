-- The CHECK constraint keeps allowing 'on_scroll'; rebuilding the table to drop it isn't worth it.
UPDATE settings SET read_mode = 'on_open' WHERE read_mode = 'on_scroll';
