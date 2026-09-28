# State Machines

State transitions are domain rules and must be centralized/tested.

## Upload
INITIATED → UPLOADING → UPLOADED → VERIFYING → PROCESSING → READY

Failure branches may include REJECTED and FAILED with explicit retry semantics. Exact persistence states will be finalized with the upload milestone.

## Space
ACTIVE → ARCHIVED → DELETION_PENDING → DELETED/PURGED

An EXPIRING presentation state may be derived or persisted depending on retention implementation. Do not silently retain data contrary to the user-facing policy.

## Membership
INVITED → ACTIVE → LEFT/REMOVED where applicable.

## Payment
PENDING → PAID | FAILED; successful payments may later transition to REFUNDED according to provider/business rules.

Invalid transitions must be rejected rather than silently coerced.
