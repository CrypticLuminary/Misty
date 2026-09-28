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
- [ ] Establish owner identity/session foundation.
- [ ] Establish scoped guest sessions without requiring a full account.
- [ ] Use secure server-side session meaning; browser state alone never authorizes.
- [ ] Define cookie/CSRF strategy for the deployed topology before mutating browser endpoints.
- [ ] Add session expiry/revocation behavior and tests.

**Gate:** an anonymous browser cannot become a member or perform protected operations without a valid server-recognized flow.

## M2.3 Space use cases
- [ ] Create Space.
- [ ] Read Space summary.
- [ ] List current member's Spaces where appropriate.
- [ ] Archive Space.
- [ ] Reject writes to non-ACTIVE Spaces unless explicitly allowed.
- [ ] Record audit events for sensitive transitions.

**Gate:** use cases are transport-independent and authorization is not embedded only in handlers.

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

## M2.5 Capability authorization
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
- [ ] Central capability evaluator.
- [ ] Role presets map to capabilities.
- [ ] Every protected use case invokes the same authorization boundary.
- [ ] Deny-by-default behavior.
- [ ] Cross-Space authorization property/invariant tests.

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
