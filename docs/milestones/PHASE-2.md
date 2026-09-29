# Phase 2 — Secure Space Access

Status: PLANNED / STARTING

## Objective
Deliver Misty's first real vertical slice: a creator can create a private Space, issue a scoped invitation, a guest can join with low friction, and every protected operation is authorized through server-side Space capabilities.

This phase intentionally comes before media upload. Upload authorization is unsafe until Space identity, membership, invitation and capability boundaries exist.

## M2.1 Domain model and invariants
- [x] Define Space lifecycle: ACTIVE → ARCHIVED → DELETING → DELETED.
- [x] Define membership lifecycle: ACTIVE → LEFT / REMOVED.
- [x] Define invitation lifecycle and expiry/revocation semantics.
- [x] Define capability vocabulary and role presets without using roles as the authorization boundary.
- [x] Add PostgreSQL constraints/indexes for cross-Space integrity and token uniqueness.
- [x] Add domain transition tests.

**Gate:** invalid transitions and cross-Space relationships are rejected by authoritative domain/database rules.

**Evidence:** `0002_secure_space_access.sql` enforces lifecycle shape, active-owner/membership uniqueness, invitation verifier constraints and same-Space invitation creators. `domain.rs` rejects invalid transitions and includes PostgreSQL integration tests for the critical constraints.

## M2.2 Identity and sessions
- [x] Establish owner identity/session foundation.
- [x] Establish scoped guest sessions without requiring a full account.
- [x] Use secure server-side session meaning; browser state alone never authorizes.
- [x] Define cookie/CSRF strategy for the deployed topology before mutating browser endpoints.
- [x] Add session expiry/revocation behavior and tests.

**Gate:** an anonymous browser cannot become a member or perform protected operations without a valid server-recognized flow.

**Evidence:** identities are neutral principals while role/display-name state is Space-membership scoped. Session issuance generates 256-bit OS-random opaque credentials, stores only 32-byte SHA-256 verifiers, rejects malformed credentials before database access, binds scoped sessions to the exact active membership identity, and denies expired, revoked, or removed access. Exact implementation commit `c5d7598451a5699fe9656257e61b787021b9f43d` passed CI `36427736526`, Security `36427736343`, and CodeQL `36427736345`.

## M2.3 Authorization kernel and Space use cases
- [x] Add the central deny-by-default capability evaluator before protected Space operations.
- [x] Make role presets map explicitly to capabilities; roles are not the authorization boundary.
- [x] Create Space and owner membership atomically.
- [x] Read Space summary through the shared authorization boundary.
- [x] List current member's Spaces where appropriate.
- [x] Archive Space through the shared authorization boundary.
- [x] Reject currently supported invalid/non-ACTIVE mutations; reusable write guard remains deferred until a real write use case consumes it.
- [x] Prevent ownerless active Spaces through currently supported use cases; owner remove/leave/transfer semantics are gated to P2-T07 before those mutations exist.
- [x] Record audit events in the same transaction boundary as sensitive transitions.
- [x] Add cross-Space negative tests for the Space operations introduced here.

**Gate:** protected Space use cases are transport-independent, deny by default, and cannot be called successfully without the same central authorization boundary.

**Evidence:** P2-T01 through P2-T04 passed their mandatory security/correctness and architecture/alignment reviews. Reviewed integration head `ae32f8c4006f75c767c5cb98389ef7f129f95e12` passed CI `36442267842`, Security `36442267720`, and CodeQL `36442267973`.

## M2.4 Invitations and joining
- [ ] Generate cryptographically unpredictable invitation secrets.
- [ ] Store only a safe verifier/hash, never a reusable raw invitation secret.
- [ ] Invitation expiry.
- [ ] Invitation revocation.
- [ ] Optional join limits/future abuse-control hooks.
- [ ] Guest enters display name and receives Space-scoped membership/session.
- [ ] Replayed/revoked/expired invitation tests.
- [ ] Avoid leaking invitation secrets in logs, analytics or error reporting.

**Gate:** possession of an asset/Space/member ID alone never grants access; invitation secrets are scoped and revocable.

## M2.5 Capability authorization hardening
M2.3 introduces the authorization kernel because protected use cases must not precede it. This milestone proves that the boundary remains complete as invitations and joining add more paths.

Initial vocabulary:
- `can_view`
- `can_upload`
- `can_download`
- `can_download_original`
- `can_delete_own`
- `can_delete_any`
- `can_invite`
- `can_manage_members`
- `can_manage_space`
- `can_enable_ai`
- `can_view_location`

Tasks:
- [ ] Verify every protected Phase 2 use case invokes the same authorization boundary.
- [ ] Add invitation/join capability cases without bypass paths.
- [ ] Add deny-by-default regression tests for newly introduced capabilities.
- [ ] Add broader cross-Space authorization property/invariant tests.
- [ ] Review the vocabulary and remove/rename presets that encode identity/account status rather than access policy.

**Gate:** frontend hiding is never required for security and authorization tests cover negative cases.

## M2.6 API contract
- [ ] Versioned REST routes for Phase 2 only.
- [ ] Stable structured error codes.
- [ ] OpenAPI generation/source of truth.
- [ ] Generate or derive frontend types/client from API contract.
- [ ] Cursor semantics established for member collections if needed.
- [ ] Idempotency policy for create/join mutations.

**Gate:** frontend does not hand-maintain conflicting request/response models.

## M2.7 Minimal web vertical slice
- [ ] Create Space screen.
- [ ] Space landing screen.
- [ ] Invitation/share surface.
- [ ] Guest join gate.
- [ ] Member view.
- [ ] Archive/revocation UI where authorized.
- [ ] Accessible loading/error/empty states.
- [ ] No fake upload/AI capability presented as working.

**Gate:** browser E2E covers creator → invite → guest join → protected Space view → revoke/deny.

## M2.8 Abuse, audit and observability
- [ ] Rate-limit invitation/join/session-sensitive paths.
- [ ] Structured audit records for create/invite/join/revoke/remove/archive.
- [ ] Correlation IDs survive use-case boundaries.
- [ ] Metrics for auth failures, invite failures and joins without logging secrets.
- [ ] Define suspicious-join hooks without premature detection machinery.

**Gate:** operators can investigate access behavior without exposing private credentials/data.

## M2.9 Phase self-review
- [ ] Correctness review.
- [ ] Authorization/threat-model review.
- [ ] Privacy review.
- [ ] Abuse review.
- [ ] Performance/query review.
- [ ] Architecture/code-quality review.
- [ ] E2E validation.
- [ ] Documentation/context-recovery review.
- [ ] CI/security evidence reviewed.
- [ ] CURRENT_STATE updated.

## Phase 2 exit criterion
Two browsers can complete the real creator/guest flow against the real API/database: create a private Space, create an expiring invitation, join as a guest, view only authorized Space data, revoke access/invitations, and observe denial afterward. Negative cross-Space tests and security gates pass. No media bytes or AI are required for this phase.
