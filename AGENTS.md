# Misty Agent Operating Manual

This file is the entry point for every human or AI contributor.

## Read order
1. `CURRENT_STATE.md`
2. `docs/PRODUCT.md`
3. `docs/ARCHITECTURE.md`
4. `docs/SECURITY.md`
5. The document for the subsystem you will change.
6. Relevant ADRs under `docs/adr/`.

Do not begin a broad refactor before understanding these documents.

## Engineering constitution
1. Correctness before cleverness.
2. Security cannot be bypassed for convenience.
3. Accepted originals are immutable.
4. Business rules have one authoritative implementation.
5. DRY knowledge, not blindly every repeated line.
6. Prefer explicit code over magical abstractions.
7. Abstract volatile external systems (storage, AI, payments), not everything.
8. Controllers stay thin; business logic belongs to use cases/domain services.
9. Background operations must tolerate retries and duplicate delivery.
10. Important state transitions are explicit and validated.
11. Database constraints defend invariants.
12. The frontend is never a security boundary.
13. AI enhances Misty; core upload/view/download must function without AI.
14. Optimize based on measurements while designing clean scale boundaries.
15. Failures should be recoverable whenever possible.
16. Logs must explain behavior without leaking private data or credentials.
17. Tests protect behavior and invariants, not implementation trivia.
18. Dependencies must justify their maintenance/security cost.
19. Simple code beats premature distributed architecture.
20. Every abstraction must justify its existence.

## Hard prohibitions
Never:
- mutate or overwrite an accepted original;
- expose private object storage publicly as an access workaround;
- authorize an operation solely in frontend code;
- log auth tokens, signed URLs, private media contents, or unnecessary sensitive metadata;
- put expensive AI inference in the synchronous upload request path;
- silently swallow failures;
- disable tests, type checks, security checks or lint rules merely to make CI green;
- weaken a type to `any`/equivalent merely to bypass a compiler;
- add microservices, Kafka, Kubernetes or a new database without an ADR and measured need;
- merge an architectural change without updating its documentation.

## Required workflow
For each milestone:
IMPLEMENT → TEST → SECURITY REVIEW → PERFORMANCE REVIEW → ARCHITECTURE REVIEW → CODE QUALITY REVIEW → DOCUMENTATION REVIEW → E2E VALIDATION.

If a gate fails, fix the defect and repeat affected gates. Do not mark a milestone complete because code was merely written.

## Definition of done
A change is done only when its acceptance criteria pass, tests are appropriate, security boundaries remain intact, documentation reflects durable decisions, CI is green, and `CURRENT_STATE.md` accurately states what is and is not complete.

## Autonomous-agent behavior
Prefer small, reviewable commits. Do not invent product requirements to unblock yourself. Record unresolved owner decisions in `CURRENT_STATE.md`. If a safe reversible implementation choice is needed, choose the simplest option consistent with existing ADRs and document it. Never represent an untested or unavailable capability as complete.
