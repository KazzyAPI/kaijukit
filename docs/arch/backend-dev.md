# Backend: run and test

Last reviewed: 2026-11-24

Nothing needs installing on the host except Docker. Rust builds run in a `rust` container with cargo caches in named volumes.

## Run the stack

```sh
docker compose -f deploy/docker-compose.yml up -d --build
curl -u admin@kaijukit.local:kk-dev-token localhost:8080/wiki/api/v2/spaces
# {"_links":{"base":"http://localhost:8080/wiki"},"results":[]}
docker compose -f deploy/docker-compose.yml down -v   # stop and wipe data
```

The server starts with `--migrate` (applies `migrations/`) and bootstraps the admin from `KK_ADMIN_EMAIL` / `KK_ADMIN_TOKEN` (defaults `admin@kaijukit.local` / `kk-dev-token`; the default token logs a warning). Probes: `/healthz` (process), `/readyz` (database). Other env: `KK_DATABASE_URL`, `KK_BIND` (default `0.0.0.0:8080`), `KK_BASE_URL` (default `http://localhost:8080`, used for `_links.base`), `RUST_LOG`.

## Test, lint, format

Tests need a Postgres superuser URL in `KK_TEST_DATABASE_URL`; each test creates its own database. Start Postgres from the compose file, then run cargo on the compose network:

```sh
docker compose -f deploy/docker-compose.yml up -d --wait postgres

cargo_in_docker() {
  docker run --rm --network kaijukit_default -v "$PWD":/w -w /w \
    -v kk-cargo:/usr/local/cargo/registry -v kk-target:/cache/target \
    -e CARGO_TARGET_DIR=/cache/target \
    -e KK_TEST_DATABASE_URL=postgres://kk:kk@postgres:5432/postgres \
    rust:1-bookworm sh -c "rustup component add rustfmt clippy >/dev/null 2>&1; $*"
}

cargo_in_docker 'cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace'
```

`kk-core` and `kk-adf` tests need no database. Each test drops its `kk_test_*` database when it ends (a killed run can leave one behind; drop it by hand).

## Layout

- `crates/kk-core`: ids, statuses, `ApiError` (pure).
- `crates/kk-adf`: ADF types, storage <-> ADF converter.
- `crates/kk-server`: axum routers (`pages`, `spaces`, `tree`, `versions`), auth, sqlx queries; `tests/api.rs` drives the HTTP API in-process against Postgres.
- `migrations/`: forward-only SQL. Assumptions made where the reference is silent: [ASSUMPTIONS.md](../spec/confluence/backend/ASSUMPTIONS.md).
