CREATE TYPE registration_mode AS ENUM ('closed', 'open');

ALTER TABLE local_instance ADD COLUMN registration_mode registration_mode NOT NULL DEFAULT 'closed';
