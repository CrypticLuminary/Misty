# Phase 2 Execution Plan — Secure Space Access

This checklist is the execution companion to `PHASE-2.md`. The milestone document defines product gates; this file defines the order of work and the review loop.

## Completion rule

A task is complete only when all four conditions are true:

1. implementation + tests satisfy the task acceptance criteria;
2. exact-head CI, Security and CodeQL are green;
3. **Review A — Security / bugs / vulnerabilities** passes or findings are fixed;
4. **Review B — Approach / alignment** confirms the abstraction, sequencing and complexity still fit Misty.

Do not mark a task complete merely because the happy path works.

### Review A — Security / bugs / vulnerabilities
For every task ask:
- Can identity, Space, membership or resource IDs be swapped across Spaces?
- Is authorization deny-by-default and server-side?
- Can inactive/revoked/expired state still authorize?
- Is there a race, replay, double-submit or retry problem?
- Can partial failure leave durable state inconsistent?
- Are secrets/private metadata exposed through logs, errors, URLs or telemetry?
- Do database constraints defend the important invariant?
- Are negative cases tested for the reason intended?
- Can the operation be abused at scale?
- Does the new code weaken an earlier invariant?

### Review B — Approach / system alignment
For every task ask:
- Is this the simplest model that represents the real product behavior?
- Is the responsibility in the correct layer?
- Does it duplicate identity/session/membership/capability state?
- Is a name encoding the wrong concept?
- Are we adding infrastructure before a real slice needs it?
- Does this keep the core journey moving forward?
- Would the next feature naturally build on this abstraction?
- Are we creating a temporary design that will immediately be replaced?
- Does this remain understandable to another team member?
- If we changed the assumption, would this design fail gracefully?

## M2.3 — Authorization kernel + Space vertical slice

### P2-T01 Central authorization kernel
- [x] Authorization accepts authenticated session context, not caller-selected identity.
- [x] Scoped sessions cannot gain ambient authority in another Space.
- [x] Active membership is required.
- [x] Role preset maps explicitly to requested capability.
- [x] Deny by default.
- [x] Cross-Space and scoped-session negative integration tests.
- [x] Review A complete.
- [x] Review B complete.

**Evidence:** implementation commit `f1afadc87d653249af24516c9caf843884d2fbbb` passed CI `36435238753`, Security `36435238811`, and CodeQL `36435238765`.

**Review A:** passed after fixing a TOCTOU risk found during review. The evaluator now uses the caller-owned PostgreSQL connection and locks the active authorizing membership with `FOR SHARE`; protected mutations must authorize and mutate in the same transaction. Tests cover scoped-session cross-Space denial, inactive membership denial, creator-provenance non-authority, role capability denial and lock behavior.

**Review B:** accepted. The boundary remains `AuthenticatedSession → active Membership → role preset → Capability`; no persisted ACL matrix, RLS policy layer or HTTP-specific authorization was introduced. This is the smallest model that supports the Phase 2 product flow and leaves later capability evolution centralized.

### P2-T02 Create Space atomically
- [x] Validate/normalize Space name.
- [x] Create Space + owner membership in one transaction.
- [x] Owner relationship is membership-based; `created_by_identity_id` is provenance only.
- [x] Prevent partial Space-without-owner creation.
- [x] Record audit evidence transactionally.
- [x] Retry/idempotency decision documented before HTTP exposure.
- [x] Review A complete.
- [x] Review B complete.

**Evidence:** implementation commit `b5d2962f314b9958f818aaf1929ef20419f57207` passed CI `36436562653`, Security `36436562849`, and CodeQL `36436562657`.

**Review A:** passed. Space, active owner membership and `space.created` audit evidence commit atomically; tests force both owner-insert failure and audit-insert failure and prove no partial Space survives. Scoped guest sessions cannot bootstrap new Spaces, names are normalized/validated before persistence, and audit actor identity/membership is protected by a same-Space composite foreign key. Audit metadata is explicitly restricted to allowlisted non-secret data. HTTP retry safety is deliberately not claimed yet; the idempotency-key contract must exist before CreateSpace is exposed in M2.6.

**Review B:** accepted. CreateSpace is a deliberate bootstrap exception to Space capability authorization because no Space membership exists before creation; requiring an identity-scoped authenticated session is the correct boundary. `created_by_identity_id` remains provenance while ownership is created as Membership state. The audit table was introduced because a real use case now requires it; no generic repository/service or extra infrastructure was added.

### P2-T03 Read + list Space
- [ ] Protected Space summary through central authorization.
- [ ] List only active memberships appropriate to the current identity/session scope.
- [ ] No ID enumeration/cross-Space leakage.
- [ ] Query/index review.
- [ ] Review A complete.
- [ ] Review B complete.

### P2-T04 Archive + ACTIVE-only mutation rules
- [ ] Archive via `can_manage_space`.
- [ ] Forward-only transition.
- [ ] Non-ACTIVE Spaces reject mutations unless explicitly documented.
- [ ] Audit archive transition in same durable boundary.
- [ ] Concurrent archive/update behavior tested.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.4 — Invitations + joining

### P2-T05 Invitation credential boundary
- [ ] 256-bit unpredictable invitation secret.
- [ ] Canonical transport encoding.
- [ ] Store verifier only.
- [ ] Creator requires `can_invite`.
- [ ] Expiry/revocation/max-use semantics.
- [ ] Atomic consumption strategy; no read-then-increment race.
- [ ] Review A complete.
- [ ] Review B complete.

### P2-T06 Guest join transaction
- [ ] Validate Space-scoped display name.
- [ ] Consume usable invite atomically.
- [ ] Create neutral identity + guest membership + scoped session atomically where appropriate.
- [ ] Replay/revoked/expired/exhausted invite denial.
- [ ] Generic safe external errors.
- [ ] Review A complete.
- [ ] Review B complete.

### P2-T07 Membership + invite revocation
- [ ] Revoke invite.
- [ ] Remove member through capability boundary.
- [ ] Revoke that membership's scoped sessions in the same operation.
- [ ] Owner cannot accidentally leave an ACTIVE Space ownerless.
- [ ] Denial is observable on the next request.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.5 — Authorization hardening

### P2-T08 Authorization matrix review
- [ ] Enumerate every Phase 2 protected use case × capability.
- [ ] Property/negative tests across two Spaces and multiple identities.
- [ ] Confirm roles are presets only.
- [ ] Confirm creator provenance is never authorization.
- [ ] Review capability names for product meaning.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.6 — API contract

### P2-T09 Versioned Phase 2 REST contract
- [ ] Stable routes and structured errors.
- [ ] Session cookie extraction only at transport boundary.
- [ ] Same-origin/CSRF enforcement on browser mutations.
- [ ] OpenAPI source of truth.
- [ ] Frontend types/client derived from the contract.
- [ ] Create/join idempotency policy implemented.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.7 — Minimal web vertical slice

### P2-T10 Creator flow
- [ ] Create Space.
- [ ] Space landing.
- [ ] Invite/share.
- [ ] Archive/revoke controls only when authorized.
- [ ] Accessible loading/error/empty states.
- [ ] Review A complete.
- [ ] Review B complete.

### P2-T11 Guest flow
- [ ] Invitation landing with safe secret handling.
- [ ] Guest display name + join.
- [ ] Protected member view.
- [ ] Removed/revoked state is handled clearly.
- [ ] No fake upload/AI affordances.
- [ ] Review A complete.
- [ ] Review B complete.

### P2-T12 Two-browser E2E
- [ ] Creator creates Space.
- [ ] Creator creates expiring invitation.
- [ ] Separate browser joins.
- [ ] Guest sees only authorized Space.
- [ ] Creator revokes access/invite.
- [ ] Guest is denied afterward.
- [ ] Cross-Space negative E2E.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.8 — Abuse, audit, observability

### P2-T13 Operational controls
- [ ] Rate limits for session/invite/join-sensitive paths.
- [ ] Structured audits for create/invite/join/revoke/remove/archive.
- [ ] Correlation ID survives handler → use case → audit/outbox.
- [ ] Safe auth/invite/join metrics without secrets.
- [ ] Suspicious-join hooks defined without premature detection machinery.
- [ ] Review A complete.
- [ ] Review B complete.

## M2.9 — Phase exit

### P2-T14 Full Phase 2 review
- [ ] Correctness.
- [ ] Authorization/threat model.
- [ ] Privacy.
- [ ] Abuse.
- [ ] Performance/query behavior.
- [ ] Architecture/code quality.
- [ ] Browser E2E.
- [ ] Documentation/context recovery.
- [ ] Exact final-head CI/Security/CodeQL evidence.
- [ ] Rust/Axum architecture validation gate.
- [ ] `CURRENT_STATE.md` updated.

## Exit criterion

Two independent browser contexts can complete the real creator/guest flow against the real API and PostgreSQL: create private Space → create expiring invitation → guest joins → authorized protected view → creator revokes → guest is denied. Cross-Space negative tests pass. No media or AI is required.
