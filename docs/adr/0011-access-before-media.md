# ADR-0011: Phase 2 establishes access control before media upload

Status: Accepted

## Decision
Misty implements Space identity, membership, invitations, sessions and capability authorization before implementing user media upload.

## Why
Upload authorization depends on who may place bytes into which Space and who may later view/download/delete them. Building upload first would either duplicate temporary authorization logic or create an unsafe public-like storage path.

The first product vertical slice therefore proves the access-control boundary end to end. Media upload becomes the next phase and reuses this boundary rather than inventing another one.

## Consequences
- Phase 2 contains no fake media feature.
- Invitation/session secrets are treated as credentials.
- Authorization lives in server-side use cases/policies, not UI conditions.
- Phase 3 can issue scoped object-storage upload capabilities only after successful Space authorization.
