# Data Model Direction

This is a design guide, not a finalized schema.

Core durable concepts: User/Identity, Space, Membership, Invitation, Asset, Upload, Derivative, ProcessingJob/OutboxEvent, FaceCluster/Embedding metadata, Moment, Download/Export, Retention state, Subscription/Entitlement and AuditEvent.

## Key invariants
- Every media Asset belongs to exactly one Space.
- Authorization is evaluated in Space context.
- An accepted original has a stable storage identity/checksum and is never overwritten.
- Derivatives reference their source Asset and processing version.
- AI artifacts record model/version information sufficient for safe reprocessing.
- Invitation secrets should not be stored in recoverable plaintext when a verifier/hash design can satisfy the use case.
- Durable state belongs in PostgreSQL; Redis must not become hidden truth.

The concrete schema will be introduced alongside the use case that needs it, with database constraints and migrations reviewed as part of that milestone.
