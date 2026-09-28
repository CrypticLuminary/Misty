# Architecture

## Shape
Start as a **modular monolith plus independent asynchronous workers**. Preserve module boundaries so measured bottlenecks can later be extracted without starting with distributed-system complexity.

```text
Web/PWA (Next.js + TypeScript)
          |
       CDN/Edge
          |
 Core API (Rust/Axum/Tokio)
    |        |         |
Postgres   Redis   Private S3-compatible storage
 +pgvector              ^
    |                   |
 transactional          | direct signed transfer
 outbox                  |
    v                    |
 job dispatch ----------+
    |
    +--> Rust media workers
    +--> Python AI workers
    +--> FFmpeg subprocesses where appropriate
```

## Control plane vs data plane
The API authorizes and records uploads/downloads. Large media bytes should normally transfer directly between the client and object storage using short-lived scoped authorization. Routing multi-gigabyte media through the API is not the default design.

## Source of truth
- PostgreSQL: durable relational truth and critical state.
- Object storage: immutable originals and regenerable derivatives.
- Redis: disposable cache/coordination/rate-limit state; never the only durable truth.
- Vector data: pgvector initially; dedicated vector infrastructure only after measured need.

## Domain modules
Accounts, Spaces, Memberships, Invitations, Media, Uploads, Downloads, Processing, Discovery, Retention, Billing, Notifications and Audit.

## API style
REST + OpenAPI initially. Generate/derive frontend contracts from the backend schema rather than hand-maintaining conflicting models.

## Realtime
Prefer SSE for one-way processing/progress updates initially. Introduce WebSockets only for demonstrated bidirectional realtime requirements.

## Reliability
Critical state changes and event creation use a transactional outbox. Job envelopes are language-neutral and versioned. Consumers are idempotent.

## External volatility
Storage, AI inference and payment providers receive narrow interfaces. Do not hide PostgreSQL behind a generic database abstraction solely for hypothetical portability.

## Performance
Use cursor pagination, virtualized galleries, lazy derivative loading, CDN delivery, asynchronous processing and measured indexing. Never use originals as gallery thumbnails.


## Complexity budget
The architecture diagram is a direction, not an instruction to activate every component early. Redis, pgvector, outbox dispatch, dedicated queues, billing integrations and additional services remain dormant until a concrete vertical slice requires them.

Keep module boundaries aligned to business capabilities, but do not extract services merely because a module exists. The modular monolith is the default until measured scaling, reliability, deployment or team-ownership needs justify extraction.

At the Phase 2 exit, explicitly review the Rust core API based on maintainability, transaction ergonomics, contributor onboarding, API-contract generation, test friction and delivery velocity before expanding into media workflows.
