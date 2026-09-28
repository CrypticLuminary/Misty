# Job System

Background work must assume at-least-once-like delivery behavior: a handler may run more than once.

## Job envelope
Use a language-neutral versioned schema containing at minimum: schema version, job/event type, entity identifiers, correlation ID and attempt/context metadata. Do not serialize language-specific objects across Rust/Python boundaries.

## Transactional outbox
When a durable database change requires asynchronous work, write the domain change and outbox record in the same PostgreSQL transaction. A dispatcher delivers pending records. Consumers are idempotent.

This prevents the classic state where the database commits `Asset=UPLOADED` but the process crashes before publishing `AssetUploaded`.

## Retry policy
Classify failures as retryable/non-retryable. Use bounded exponential backoff/jitter where appropriate. Poison jobs require visibility/dead-letter handling rather than infinite hot loops.

## Idempotency
Derivative identity should incorporate source asset + derivative kind + pipeline version where practical. Re-running a job must not corrupt or multiply durable state.
