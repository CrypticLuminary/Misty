# ADR-0005: Immutable originals

Status: Accepted / product invariant

## Decision
Once an uploaded original is accepted, Misty never modifies or overwrites its bytes.

## Consequence
All display optimization, metadata sanitation, transcoding and AI processing occurs on derivatives/indexes. Replacement means creating a new source asset/version according to future product semantics, not mutating the stored original.
