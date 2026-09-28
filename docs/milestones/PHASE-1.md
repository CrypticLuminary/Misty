# Phase 1 — Engineering Foundation

Status: IN PROGRESS

## Objective
Create a secure, reproducible foundation on which Misty's first vertical slice can be built without architectural drift.

## M1.1 Project brain and governance
- [x] Define product and invariants.
- [x] Define agent operating rules.
- [x] Define baseline architecture/security/engineering guidance.
- [x] Define roadmap.
- [ ] Add ADRs for foundational decisions.
- [ ] Add contribution/PR templates.
- [ ] Verify a fresh agent can determine current state from repo docs alone.

**Gate:** documentation has no known contradiction about core invariants; CURRENT_STATE accurately describes reality.

## M1.2 Monorepo skeleton
- [ ] Scaffold `apps/web` Next.js + strict TypeScript.
- [ ] Scaffold `apps/api` Rust/Axum/Tokio.
- [ ] Scaffold `workers/media` Rust boundary.
- [ ] Scaffold `workers/ai` Python boundary.
- [ ] Add shared contract/config locations without premature abstractions.
- [ ] Add root developer commands/documentation.
- [ ] Pin/lock dependencies appropriately.

**Gate:** each component builds/lints/tests independently; no placeholder is represented as a working product capability.

## M1.3 Reproducible local infrastructure
- [ ] PostgreSQL with pgvector.
- [ ] Redis.
- [ ] S3-compatible local object storage.
- [ ] Docker Compose orchestration.
- [ ] Health checks.
- [ ] Example environment file containing no secrets.
- [ ] Document startup/reset/debug workflow.

**Gate:** a fresh environment can start dependencies through the documented workflow and health checks pass.

## M1.4 API/worker foundation
- [ ] API health/readiness endpoints.
- [ ] Structured logging and correlation/request IDs.
- [ ] Configuration validation at startup.
- [ ] Database connectivity/migration foundation.
- [ ] Versioned language-neutral job envelope.
- [ ] Worker health/smoke behavior.
- [ ] Transactional-outbox schema/design foundation (no fake delivery guarantee).

**Gate:** smoke/integration tests exercise real local dependencies where practical and failure states are explicit.

## M1.5 CI and security baseline
- [ ] Web format/lint/type/test/build jobs.
- [ ] Rust fmt/clippy/test/audit jobs.
- [ ] Python lint/type/test jobs.
- [ ] Secret scanning.
- [ ] Dependency review/audit.
- [ ] CodeQL/static analysis where supported.
- [ ] Least-privilege GitHub Actions permissions.
- [ ] Pin third-party Actions to immutable commit SHAs where practical.
- [ ] Container scan once application images exist.
- [ ] No production secrets in CI.

**Gate:** intentionally broken lint/test examples fail CI; no workflow requires write permissions unless its job needs them.

## M1.6 Foundation self-review
Run the complete loop:
- [ ] Correctness review.
- [ ] Security/threat-boundary review.
- [ ] Performance/obvious-footgun review.
- [ ] Architecture review.
- [ ] Code-quality/dependency review.
- [ ] Documentation/context-recovery review.
- [ ] Reproducibility review.
- [ ] CI evidence reviewed.
- [ ] Known limitations recorded.
- [ ] CURRENT_STATE updated.

**Phase 1 exit criterion:** another contributor/agent can clone Misty, understand the product/invariants, start the documented environment, run quality gates, observe health checks and safely begin Phase 2 without relying on chat history.
