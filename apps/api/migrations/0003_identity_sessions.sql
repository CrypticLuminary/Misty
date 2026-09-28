-- Server-recognized identity and session foundation for Phase 2.

CREATE TABLE identities (
    id UUID PRIMARY KEY,
    display_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        display_name IS NULL
        OR char_length(btrim(display_name)) BETWEEN 1 AND 80
    )
);

ALTER TABLE spaces
    ADD CONSTRAINT spaces_creator_identity_fk
    FOREIGN KEY (created_by_identity_id) REFERENCES identities(id);

ALTER TABLE memberships
    ADD CONSTRAINT memberships_identity_fk
    FOREIGN KEY (identity_id) REFERENCES identities(id);

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identities(id),
    secret_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(secret_hash) = 32),
    space_id UUID REFERENCES spaces(id),
    membership_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK (
        (space_id IS NULL AND membership_id IS NULL)
        OR (space_id IS NOT NULL AND membership_id IS NOT NULL)
    ),
    FOREIGN KEY (space_id, membership_id)
        REFERENCES memberships (space_id, id)
);

CREATE INDEX sessions_identity_active_idx
    ON sessions (identity_id, expires_at)
    WHERE revoked_at IS NULL;

CREATE INDEX sessions_space_active_idx
    ON sessions (space_id, expires_at)
    WHERE revoked_at IS NULL AND space_id IS NOT NULL;
