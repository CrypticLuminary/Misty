CREATE TABLE account_bootstrap_tokens (
    id UUID PRIMARY KEY,
    secret_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CHECK (expires_at > created_at)
);

COMMENT ON TABLE account_bootstrap_tokens IS
'Pre-production bootstrap mechanism only. Replace with real account identity provider before deployment.';
