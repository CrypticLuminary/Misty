# ADR-0002: Rust core API

Status: Provisional — validate in first vertical slice

## Decision
Use Rust with Axum/Tokio as the initial core API candidate.

## Why
The control plane will handle high-concurrency authorization, upload/download orchestration and event/job coordination. Rust provides memory safety, strong types and predictable resource use.

## Guardrail
This is not a vanity-language decision. The secure original-upload vertical slice must validate developer ergonomics, ecosystem integration and maintainability before the choice is considered locked.
