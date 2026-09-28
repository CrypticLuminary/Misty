# Development

## Prerequisites
Git, Docker with Compose, Rust stable, Node.js 24 + pnpm 10.17.1, and Python 3.12+ with uv 0.8.17.

## Local dependencies
Copy `.env.example` to `.env` for local development only. Docker Compose reads `.env` automatically. Host-run Rust/Python processes do not, so export the needed variables into your shell before starting them (for Bash: `set -a; . ./.env; set +a`; for PowerShell, set the corresponding `$env:...` values). Then:

```sh
docker compose pull
docker compose up -d
docker compose ps
```

This starts the pinned PostgreSQL/pgvector, Redis and MinIO services. The example credentials are local-only and must not be reused in production.

All three services should report healthy before starting the API. To inspect failures:

```sh
docker compose logs -f postgres redis minio
```

To stop without deleting local data:

```sh
docker compose down
```

To perform a destructive local reset, including database/object/cache volumes:

```sh
docker compose down -v
docker compose up -d
```

## Components
Install/sync dependencies from committed lockfiles and run components with locked resolution:

```sh
cargo run --locked -p misty-api
cargo run --locked -p misty-media-worker

pnpm install --frozen-lockfile
cd apps/web
pnpm dev

uv sync --project workers/ai --frozen
PYTHONPATH=workers/ai/src uv run --project workers/ai python -m misty_ai.main
```

The API requires `DATABASE_URL` and accepts `MISTY_BIND_ADDR` (default `0.0.0.0:8080`). With the example environment and API running:

```sh
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/ready
```

`/health` reports process health. `/ready` also verifies PostgreSQL connectivity. Startup applies the checked-in SQL migrations.

## Quality gates
Start the local PostgreSQL service before Rust tests because the outbox integration test uses the real migration.

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features

pnpm install --frozen-lockfile
cd apps/web
pnpm lint
pnpm typecheck
pnpm test
pnpm build

cd ../..
uv sync --project workers/ai --frozen
uv run --project workers/ai ruff check workers/ai
uv run --project workers/ai mypy workers/ai/src
PYTHONPATH=workers/ai/src uv run --project workers/ai pytest workers/ai/tests
```

CI additionally boots the full Docker Compose dependency set, checks service health, validates configuration failure behavior, runs the media-worker smoke path, applies API migrations, and verifies API health/readiness plus request-ID propagation.

CI is authoritative for the repository state. If documentation and CI disagree, fix the disagreement rather than bypassing a gate.

## Current limitations
This is still an engineering foundation, not a product implementation. The API currently connects to PostgreSQL and applies migrations, but product modules, Redis integration, object-storage integration, upload/download flows, media processing, AI discovery, retention and billing are not implemented. See `CURRENT_STATE.md`.
