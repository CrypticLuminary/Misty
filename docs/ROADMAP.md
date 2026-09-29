# Misty Roadmap

## Phase 1 — Engineering Foundation
See `docs/milestones/PHASE-1.md`. Establish project brain, monorepo boundaries, reproducible local services, CI/security gates, health checks, typed contracts and observability baseline.

## Phase 2 — Spaces, identity and authorization
Registered/guest identity, Spaces, memberships, invitations, capability authorization and audit events.

## Architecture validation gate after Phase 2
Before media work expands the system, review the first real browser → API → PostgreSQL vertical slice. Confirm that Rust/Axum is improving clarity and reliability enough to justify its team/onboarding cost; review transaction ergonomics, OpenAPI/frontend-contract generation, test friction and contributor velocity. Continue with Rust if the evidence is good; change direction here rather than after media workflows depend on it.

## Phase 3 — Secure original upload
Direct private object-storage upload, resumability/multipart where justified, quarantine/verification, checksums, idempotent completion and orphan cleanup.

## Phase 4 — Media processing and gallery
Thumbnails/previews, metadata, video derivatives, cursor pagination, virtualization, CDN strategy and upload UX.

## Phase 5 — Original retrieval and baseline retention
Authorized original downloads, selected-item retrieval, checksum verification, transparent archive/expiry behavior and complete deletion of the media artifacts introduced so far. Close the basic upload → browse → retrieve → expire loop before adding intelligence.

## Phase 6 — Group collaboration
Concurrent contributors, member controls, selections/favorites, activity and abuse controls.

## Phase 7 — Deterministic organization
Time/location clustering, exact/near duplicates, bursts, technical quality signals and Moments.

## Phase 8 — AI intelligence
Versioned Python inference, face detection/embeddings/clustering, Space-scoped Find Me, semantic embeddings/search and measured evaluation.

## Phase 9 — Advanced export, retention extensions and monetization
Batch/Find-Me export, asynchronous ZIP/export jobs, restore/extension flows, storage lifecycle tuning, billing/entitlements and the richer ACTIVE → EXPIRING → ARCHIVED → DELETION_PENDING → PURGED product lifecycle.

## Phase 10 — Hardening
Load/failure testing, authorization fuzzing, malformed media, recovery, backup restore drills, p95/p99 performance work and cost measurement.

## Phase 11 — Production readiness
Accessibility, browser/network matrix, privacy/retention disclosures, operational runbooks, alerts, incident response and end-to-end acceptance.
