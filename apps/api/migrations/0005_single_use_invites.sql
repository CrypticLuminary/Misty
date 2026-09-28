UPDATE invitations SET max_uses = 1 WHERE max_uses IS NULL;

ALTER TABLE invitations
    ALTER COLUMN max_uses SET DEFAULT 1,
    ALTER COLUMN max_uses SET NOT NULL;
