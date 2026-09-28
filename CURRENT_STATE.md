# Current State

Last updated: 2026-09-28

## Current phase
**Phase 2 — Secure Space Access (active)**

## Current milestone
**M2.3 — Authorization kernel and Space use cases**

## Completed and verified
- Product invariants, architecture, security guidance, engineering rules, roadmap and foundational ADRs.
- Next.js/strict-TypeScript web boundary, Rust API and media-worker boundaries, and Python AI-worker boundary.
- Committed Cargo, pnpm and uv lockfiles with frozen/locked CI resolution.
- Pinned Rust toolchain, runner generation, GitHub Actions and local infrastructure versions.
- PostgreSQL + pgvector, Redis and local S3-compatible storage through Docker Compose with health checks.
- API configuration validation, PostgreSQL connectivity/migrations, /health, /ready, structured logs and request-ID propagation.
- Versioned job envelope plus a transaction-bound outbox enqueue foundation tested against PostgreSQL.
- Web format/lint/type/test/build, Rust fmt/clippy/test/audit, Python lint/type/test, secret scanning and CodeQL.
- Phase 1 correctness, security, performance, architecture, dependency, documentation and reproducibility review.
- Phase 2 M2.1 domain invariants: lifecycle models, explicit capability presets, PostgreSQL cross-Space constraints and negative tests.
- Phase 2 M2.2 identity/session foundation: neutral identities, Space-scoped membership aliases/roles, 256-bit opaque session issuance, verifier-only storage, scoped membership binding, expiry/revocation and removed-member denial.

M2.2 gate evidence on commit `c5d7598451a5699fe9656257e61b787021b9f43d`: CI `36427736526`, Security `36427736343`, and CodeQL `36427736345` all passed.

Implementation evidence on commit c8cd50f183a9489757d3e5dfe49075f3a9a8fed1: CI run 36385011447, Security run 36385011516, and CodeQL run 36385011418 all passed.

## Final Phase 1 gate
Passed on commit `4023d932f471d479b6f2d954b1f72a0885b915f2`: CI run `36385462155`, Security run `36385462159`, and CodeQL run `36385462173` all completed successfully.

## Not implemented yet
No production application capability should be assumed. The following remain future work:
- invitation joining, Space use cases and capability authorization;
- application integration with Redis and object storage;
- original-media upload/download workflows and signed capabilities;
- media validation, derivatives/transcodes and gallery behavior;
- deterministic organization and AI discovery/Find Me;
- export, retention/deletion workflows and billing;
- production deployment/provider selection and application container images.

The transactional-outbox foundation does not yet include a dispatcher, queue delivery or consumers. The Rust core API remains provisional until the first secure vertical slice validates maintainability and ecosystem fit.

## Active Phase 2 milestone
**M2.3 — Authorization kernel and Space use cases.** Establish the deny-by-default capability boundary first, then implement transport-independent create/read/list/archive operations with owner-integrity, ACTIVE-only writes, cross-Space negative tests and transactional audit evidence before exposing HTTP routes.

## Owner decisions pending
None required to close the engineering foundation. Production hosting, object-storage provider and related cost/geography decisions remain intentionally deferred until measured requirements exist.
