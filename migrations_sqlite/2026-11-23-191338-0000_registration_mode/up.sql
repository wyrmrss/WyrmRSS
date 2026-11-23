ALTER TABLE local_instance ADD COLUMN registration_mode TEXT NOT NULL DEFAULT 'closed'
  CHECK (registration_mode IN ('closed', 'open'));
