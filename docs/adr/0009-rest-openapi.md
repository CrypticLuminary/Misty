# ADR-0009: REST + OpenAPI initially

Status: Accepted

## Decision
Use REST for the initial product API and OpenAPI as the machine-readable contract.

## Why
Misty's early use cases map cleanly to explicit resource/use-case endpoints. OpenAPI enables generated clients/types and reduces frontend/backend contract drift without introducing GraphQL authorization/cache complexity prematurely.
