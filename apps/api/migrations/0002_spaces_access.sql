CREATE TYPE space_status AS ENUM ('active', 'archived', 'deleting', 'deleted');
CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'contributor', 'viewer');
CREATE TYPE membership_status AS ENUM ('active', 'left', 'removed');

CREATE TABLE spaces (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 120),
    status space_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at TIMESTAMPTZ,
    CHECK ((status = 'active' AND archived_at IS NULL) OR status <> 'active')
);

CREATE TABLE memberships (
    id UUID PRIMARY KEY,
    space_id UUID NOT NULL REFERENCES spaces(id),
    subject_id UUID NOT NULL,
    role membership_role NOT NULL,
    status membership_status NOT NULL DEFAULT 'active',
    display_name TEXT NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 80),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ,
    UNIQUE (space_id, subject_id),
    CHECK ((status = 'active' AND ended_at IS NULL) OR status <> 'active')
);

CREATE INDEX idx_memberships_subject_active
    ON memberships (subject_id, space_id)
    WHERE status = 'active';

CREATE TABLE invitations (
    id UUID PRIMARY KEY,
    space_id UUID NOT NULL REFERENCES spaces(id),
    token_hash BYTEA NOT NULL UNIQUE,
    role membership_role NOT NULL CHECK (role <> 'owner'),
    created_by_membership_id UUID NOT NULL REFERENCES memberships(id),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    max_uses INTEGER CHECK (max_uses IS NULL OR max_uses > 0),
    use_count INTEGER NOT NULL DEFAULT 0 CHECK (use_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at > created_at),
    CHECK (max_uses IS NULL OR use_count <= max_uses)
);

CREATE INDEX idx_invitations_space_active
    ON invitations (space_id, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    space_id UUID REFERENCES spaces(id),
    actor_subject_id UUID,
    event_type TEXT NOT NULL CHECK (char_length(event_type) > 0),
    correlation_id UUID NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_space_created
    ON audit_events (space_id, created_at DESC);
