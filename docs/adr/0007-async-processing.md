# ADR-0007: Asynchronous derivative and AI processing

Status: Accepted

## Decision
Upload acceptance must not wait for expensive derivative/AI pipelines. Processing is asynchronous and independently retryable.

## Consequence
The UI and domain model must represent processing states honestly. AI outages must not break core storage/download capability.
