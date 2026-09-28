CREATE TYPE subject_kind AS ENUM ('account', 'guest');

CREATE TABLE subjects (
    id UUID PRIMARY KEY,
    kind subject_kind NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE memberships
    ADD CONSTRAINT fk_memberships_subject
    FOREIGN KEY (subject_id) REFERENCES subjects(id);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    subject_id UUID NOT NULL REFERENCES subjects(id),
    secret_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    CHECK (expires_at > created_at)
);

CREATE INDEX idx_sessions_subject_active
    ON sessions (subject_id, expires_at)
    WHERE revoked_at IS NULL;
