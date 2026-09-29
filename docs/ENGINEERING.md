# Engineering Practices

## Priority
Correctness → Security → Clarity → Testability → Performance → Reuse → Brevity.

## DRY
DRY duplicated **knowledge**, not every repeated line. Authorization policy, retention rules and state transitions need authoritative implementations. Two superficially similar use cases may remain separate until their shared abstraction is stable.

## Vertical-slice delivery
After the secure foundation, prefer thin end-to-end increments that leave the system in a usable, testable state. Do not implement a business operation first and bolt authorization, audit, abuse controls or transport semantics onto it several milestones later.

A vertical slice should normally include the minimum domain rule, authorization boundary, persistence, audit/observability, transport contract and negative tests needed for that operation. Cross-cutting systems should grow only when a real slice requires them.

## Use-case orientation
HTTP handlers validate transport concerns, call an application use case, and serialize a response. Avoid giant controllers and giant catch-all service classes.

Examples: `CreateSpace`, `JoinSpace`, `InitiateUpload`, `CompleteUpload`, `RequestDownload`, `FindMyPhotos`.

## State machines
Model important lifecycles explicitly and reject invalid transitions. Do not scatter arbitrary string-status checks throughout handlers.

## Idempotency
Assume requests/jobs can be delivered twice. Upload completion, derivatives, embeddings, ZIPs, deletion, notifications and payment webhooks must be safe under retry.

## Transactions and integrity
Use database transactions for multi-step durable changes. Use foreign keys, unique constraints, NOT NULL/check constraints and indexes to defend invariants at the database boundary.

## Types and contracts
TypeScript uses strict mode; avoid casual `any`. Rust uses compiler-enforced domain types where they improve correctness. Python workers use type checking where practical. API/job contracts are versioned.

## Dependencies
Prefer mature focused libraries. Every new dependency carries supply-chain, maintenance, binary-size and upgrade cost. Record major architectural dependencies in ADRs.

## Comments
Explain *why* a non-obvious constraint exists, not what obvious syntax does.

## Performance
Measure before invasive optimization. Add indexes/caches/denormalization only with a clear query/workload reason and tests/metrics where appropriate.

## PR discipline
Keep changes cohesive and reviewable. A PR should have explicit acceptance criteria and should not mix unrelated refactors with feature work.
