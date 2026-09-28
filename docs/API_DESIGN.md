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
