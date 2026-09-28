# Observability

Use structured, privacy-conscious telemetry.

## Correlation
A request/correlation ID should follow meaningful operations across API, database/outbox and workers where possible.

## Metrics direction
Upload success/speed, API p50/p95/p99, DB latency/pool pressure, queue depth, worker throughput/failures, derivative latency, storage failures, CDN hit ratio, AI latency/cost, export latency and client errors.

## Logs
Prefer fields such as event, request_id, space_id, asset_id, stage and reason. Avoid secrets, tokens, signed URLs, raw media and unnecessary sensitive metadata.

## Tracing
Prefer OpenTelemetry-compatible instrumentation so the backend provider can change without rewriting the application.
