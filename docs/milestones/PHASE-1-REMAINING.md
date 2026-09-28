# Phase 1 Remaining Work

This checklist is evidence-driven. Do not mark an item complete unless the repository or CI provides evidence.

## A. Reproducibility
- [ ] Generate and commit `Cargo.lock` from the workspace.
- [ ] Generate and commit root `pnpm-lock.yaml`.
- [ ] Generate and commit `workers/ai/uv.lock`.
- [ ] Change Rust CI/audit commands to locked dependency resolution where supported.
- [ ] Change web CI/security installs to `--frozen-lockfile`.
- [ ] Change Python CI from ephemeral `uvx` resolution to the committed environment with `uv sync --frozen` and `uv run`.
- [x] Pin local PostgreSQL/pgvector, Redis, and MinIO image versions.
- [x] Pin CI PostgreSQL/pgvector image version.
- [ ] Verify pinned image tags exist and local health checks remain valid.

## B. CI and supply-chain hardening
- [ ] Review every GitHub Action reference for immutable SHA pinning.
- [ ] Replace Node 20-runtime Action revisions with current Node 24-compatible immutable revisions where available.
- [ ] Verify least-privilege workflow permissions.
- [ ] Verify dependency audit, secret scan, and CodeQL all pass.
- [ ] Verify Web lint/type/test/build, Rust fmt/clippy/test, Python lint/type/test all pass.
- [ ] Verify PostgreSQL migration and API `/ready` smoke test pass.

## C. Foundation behavior
- [ ] Verify API health/readiness behavior and startup config validation.
- [ ] Verify structured logging/request IDs.
- [ ] Verify transactional-outbox migration and versioned job envelope.
- [ ] Verify media worker smoke behavior.
- [ ] Verify local PostgreSQL + pgvector, Redis, and S3-compatible storage health checks.
- [ ] Verify documented startup/reset/debug workflow against repository configuration.

## D. Documentation and context recovery
- [ ] Reconcile `docs/milestones/PHASE-1.md` with actual evidence.
- [ ] Verify ADRs and contribution/PR guidance exist and match implementation.
- [ ] Check product/security/architecture docs for contradictions.
- [ ] Verify a fresh contributor can determine current phase, implemented capabilities, and limitations without chat history.
- [ ] Update `CURRENT_STATE.md` only after validation.

## E. Phase 1 self-review
- [ ] Correctness review.
- [ ] Security/threat-boundary review.
- [ ] Performance/obvious-footgun review.
- [ ] Architecture review.
- [ ] Code-quality/dependency review.
- [ ] Documentation/context-recovery review.
- [ ] Reproducibility review.
- [ ] CI evidence review.
- [ ] Record known limitations.
- [ ] Run final CI + Security + CodeQL on the Phase 1 completion commit.

## Exit gate
Phase 1 is complete only when another contributor can clone Misty, understand its invariants, start the documented environment, run deterministic quality gates, observe health checks, and safely begin Phase 2 without relying on conversation history.
