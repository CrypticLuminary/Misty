# Testing Strategy

Coverage percentage is not the goal; invariant confidence is.

## Layers
- Unit: domain rules/state transitions.
- Integration: PostgreSQL, Redis, S3-compatible storage and worker boundaries.
- API: transport/authorization behavior.
- E2E: critical user journeys.
- Load: gallery/upload/worker behavior at representative scale.
- Security: cross-Space access, invitation/session abuse, malformed uploads and authorization.

## Critical invariant tests
- Viewer cannot perform disallowed mutations.
- Revoked invitation cannot create a valid membership.
- Asset from Space A cannot be authorized through Space B.
- Derivative processing never overwrites original.
- Duplicate job execution is safe.
- Permanent deletion eventually removes all required derivatives/indexes.
- Downloaded original checksum equals accepted uploaded checksum.

## Foundation phase
CI must at least prove formatting/lint/type/test/build health for each introduced language and smoke-test the runnable foundations. Do not pretend unavailable external infrastructure was tested.
