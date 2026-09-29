-- Bind scoped sessions to the exact identity + membership tuple.
-- A Space match alone is insufficient because memberships are authority-bearing records.

CREATE UNIQUE INDEX memberships_space_id_identity_unique
    ON memberships (space_id, id, identity_id);

ALTER TABLE sessions
    ADD CONSTRAINT sessions_membership_identity_fk
    FOREIGN KEY (space_id, membership_id, identity_id)
    REFERENCES memberships (space_id, id, identity_id);

ALTER TABLE sessions
    ADD CHECK (revoked_at IS NULL OR revoked_at >= created_at);
