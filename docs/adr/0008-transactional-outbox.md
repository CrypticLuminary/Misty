# ADR-0008: Transactional outbox

Status: Accepted

## Decision
For database state changes that require reliable asynchronous follow-up, write an outbox event in the same PostgreSQL transaction and dispatch it asynchronously.

## Why
Avoid committing domain state and then losing required processing because publishing failed between two independent systems.
