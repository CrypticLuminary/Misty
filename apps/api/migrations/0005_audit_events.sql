-- Durable, privacy-conscious audit evidence for Phase 2 security-sensitive actions.

CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    space_id UUID NOT NULL REFERENCES spaces(id),
    actor_identity_id UUID NOT NULL REFERENCES identities(id),
    actor_membership_id UUID NOT NULL,
    event_type TEXT NOT NULL CHECK (
        char_length(event_type) BETWEEN 1 AND 80
        AND event_type = lower(event_type)
    ),
    correlation_id UUID NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(details) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (space_id, actor_membership_id, actor_identity_id)
        REFERENCES memberships (space_id, id, identity_id)
);

CREATE INDEX audit_events_space_created_idx
    ON audit_events (space_id, created_at, id);
