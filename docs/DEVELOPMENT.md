# Development

## Prerequisites
Git, Docker with Compose, Rust stable, Node.js 24 + pnpm 10, and Python 3.12+ with uv.

## Local dependencies
Copy `.env.example` to `.env` for local development only, then:

```sh
docker compose up -d
docker compose ps
```

This starts PostgreSQL/pgvector, Redis and MinIO. These credentials are deliberately local-only examples.

## Components
```sh
cargo run -p misty-api
cargo run -p misty-media-worker

cd apps/web
pnpm install
pnpm dev

PYTHONPATH=workers/ai/src uv run --project workers/ai python -m misty_ai.main
```

## Quality gates
```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

cd apps/web
pnpm lint
pnpm typecheck
pnpm test
pnpm build

uvx ruff check workers/ai
uvx mypy workers/ai/src
PYTHONPATH=workers/ai/src uvx pytest workers/ai/tests
```

CI is authoritative for the repository state. If documentation and CI disagree, fix the disagreement rather than bypassing a gate.

## Current limitations
The foundation does not yet connect the API to PostgreSQL/Redis/object storage and does not implement product features. See `CURRENT_STATE.md`.
