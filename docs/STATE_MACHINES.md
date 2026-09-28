# State Machines

State transitions are domain rules and must be centralized/tested.

## Upload
INITIATED → UPLOADING → UPLOADED → VERIFYING → PROCESSING → READY

Failure branches may include REJECTED and FAILED with explicit retry semantics. Exact persistence states will be finalized with the upload milestone.

## Space
ACTIVE → ARCHIVED → DELETING → DELETED

An EXPIRING presentation state may be derived or persisted depending on retention implementation. Do not silently retain data contrary to the user-facing policy.

## Membership
ACTIVE → LEFT | REMOVED. Invitation state is separate from membership: a membership exists only after a join succeeds.

## Payment
PENDING → PAID | FAILED; successful payments may later transition to REFUNDED according to provider/business rules.

Invalid transitions must be rejected rather than silently coerced.


## Invitation
ACTIVE → REVOKED | EXHAUSTED. Expiry is time-based: an ACTIVE invitation whose `expires_at` is in the past is unusable without requiring a state mutation. Revocation is terminal. `EXHAUSTED` is terminal when a bounded invitation reaches its use limit.

Phase 2 deliberately uses `DELETING` as the Space deletion-in-progress state. The richer retention lifecycle in Phase 9 may add user-facing expiry/deletion-pending semantics without weakening this authorization boundary.
