# Misty

Misty is a temporary shared memory space for groups to upload original-quality photos and videos, organize them into people, moments and places, and quickly take home the media that matters to them.

## Core promise
**Store originals unchanged. Transform only derivatives. Organize asynchronously. Share securely.**

## Status
Pre-alpha. Phase 1 foundation is in progress. See [CURRENT_STATE.md](CURRENT_STATE.md), [AGENTS.md](AGENTS.md), and [docs/ROADMAP.md](docs/ROADMAP.md).

## Planned stack
- Web: Next.js + TypeScript
- Core API: Rust + Axum + Tokio
- Data: PostgreSQL + pgvector
- Ephemeral coordination: Redis
- Media: private S3-compatible object storage; libvips-class image processing; FFmpeg video processing
- AI: Python workers using the PyTorch/ONNX ecosystem
- Contracts: REST + OpenAPI; language-neutral versioned job envelopes
- Local development: Docker Compose
- CI/CD: GitHub Actions

Architecture choices are provisional until validated by the first secure vertical slice. See `docs/adr/`.

## Non-negotiable invariant
An accepted original media object's bytes are immutable. Previews, thumbnails, transcodes, embeddings and indexes are derivatives and may be regenerated.
