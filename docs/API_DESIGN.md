# API Design

Misty starts with REST + OpenAPI.

## Conventions
- Resource identifiers are opaque.
- Authorization is evaluated server-side for every protected use case.
- Errors use stable machine-readable codes plus safe human messages.
- Pagination is cursor-based for large collections.
- Idempotency is explicit on operations vulnerable to retries.
- Upload/download authorization returns short-lived scoped capabilities rather than permanent public object URLs.
- Breaking API changes require versioning/migration strategy.

Transport handlers should not own business rules. They validate transport input, invoke a use case and serialize its result.


## Idempotency policy
Operations that create durable resources from browser/API retries, including Create Space and Join Space, require explicit idempotency before their HTTP routes are exposed. The intended contract is a client-generated opaque `Idempotency-Key` scoped to the authenticated principal and operation. The server persists the first terminal result and returns the same semantic result for a safe retry with the same key.

The transport-independent Phase 2 use cases may exist before this HTTP mechanism, but they must not be exposed as retryable browser mutations until M2.6 implements the policy. Do not infer idempotency from resource names or silently deduplicate unrelated user actions.
