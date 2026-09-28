CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS outbox_events (
    id UUID PRIMARY KEY,
    schema_version SMALLINT NOT NULL CHECK (schema_version > 0),
    event_type TEXT NOT NULL CHECK (length(event_type) > 0),
    aggregate_type TEXT NOT NULL CHECK (length(aggregate_type) > 0),
    aggregate_id UUID NOT NULL,
    correlation_id UUID NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_outbox_dispatch
    ON outbox_events (available_at, created_at)
    WHERE published_at IS NULL;
