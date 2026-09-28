# Phase 1 Remaining Work

This checklist is evidence-driven. Items are checked only when repository state or CI provides evidence.

## A. Reproducibility
- [x] Generate and commit Cargo.lock from the workspace.
- [x] Generate and commit root pnpm-lock.yaml.
- [x] Generate and commit workers/ai/uv.lock.
- [x] Change Rust CI commands to locked dependency resolution where supported.
- [x] Change web CI/security installs to --frozen-lockfile.
- [x] Change Python CI from ephemeral uvx resolution to the committed environment with uv sync --frozen and uv run.
- [x] Pin local PostgreSQL/pgvector, Redis, and S3-compatible object-storage image versions.
- [x] Pin CI PostgreSQL/pgvector image version.
- [x] Verify pinned image tags exist and local health checks pass.
- [x] Pin the Rust toolchain and GitHub runner generation used by quality gates.

## B. CI and supply-chain hardening
- [x] Review every GitHub Action reference for immutable SHA pinning.
- [x] Replace Node 20-runtime Action revisions with current Node 24-compatible immutable revisions where available.
- [x] Verify least-privilege workflow permissions.
- [x] Verify dependency audit, secret scan, and CodeQL pass.
- [x] Verify Web format/lint/type/test/build, Rust fmt/clippy/test, Python lint/type/test pass.
- [x] Verify PostgreSQL migration and API /ready smoke test pass.

## C. Foundation behavior
- [x] Verify API health/readiness behavior and startup config validation.
- [x] Verify structured logging/request-ID propagation.
- [x] Verify transactional-outbox migration and versioned job envelope against PostgreSQL.
- [x] Verify media worker smoke behavior.
- [x] Verify local PostgreSQL + pgvector, Redis, and S3-compatible storage health checks.
- [x] Verify documented startup/reset/debug workflow against repository configuration.

## D. Documentation and context recovery
- [x] Reconcile docs/milestones/PHASE-1.md with actual evidence.
- [x] Verify ADRs and contribution/PR guidance exist and match implementation.
- [x] Check product/security/architecture docs for contradictions.
- [x] Verify a fresh contributor can determine current phase, implemented capabilities, and limitations without chat history.
- [x] Update CURRENT_STATE.md for the completion candidate.

## E. Phase 1 self-review
- [x] Correctness review.
- [x] Security/threat-boundary review.
- [x] Performance/obvious-footgun review.
- [x] Architecture review.
- [x] Code-quality/dependency review.
- [x] Documentation/context-recovery review.
- [x] Reproducibility review.
- [x] CI evidence review.
- [x] Record known limitations.
- [ ] Run CI + Security + CodeQL on the Phase 1 completion-state documentation commit.

## Exit gate
Phase 1 is complete only when another contributor can clone Misty, understand its invariants, start the documented environment, run deterministic quality gates, observe health checks, and safely begin Phase 2 without relying on conversation history.
