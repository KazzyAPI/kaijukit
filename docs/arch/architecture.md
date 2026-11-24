# Architecture

Last reviewed: 2026-11-24 · decisions: [ADR 0002](../decisions/0002-own-stack.md)

## Repo layout (target; M0 creates the ones marked *)

```
Cargo.toml                  workspace *
crates/
  kk-core/                  domain types, ids, statuses, errors; no IO *
  kk-adf/                   ADF types, validation, storage<->ADF conversion (native + WASM) *
  kk-cql/ kk-perm/          query parser, permission evaluation (later milestones)
  kk-server/                axum app: routers, DTOs, auth, sqlx repos, migrations *
  kk-worker/                job runner binary (M3+)
migrations/                 sqlx migrations, forward only *
web/
  package.json              pnpm workspace *
  packages/ui/              @kaijukit/ui component kit *
  packages/editor/          @kaijukit/editor (ProseMirror/Tiptap) *
  app/                      Vite React app *
conformance/
  runner/                   TypeScript runner (diff and schema modes) *
  cases/<id>/case.json      recorded cases
deploy/
  Dockerfile, docker-compose.yml *   local dev: postgres + server + web
  helm/kaijukit/            chart (M0 skeleton)
```

## Rules

- Business rules live in `kk-core`/`kk-adf` (pure, unit-tested). `kk-server` only wires HTTP, auth, and SQL.
- HTTP DTOs are separate types from domain types; a mapping function per resource. DTO field names are the wire contract and are checked against the pinned OpenAPI.
- IDs: `i64` from one Postgres sequence; JSON as strings in v2. Types: `SpaceId`, `ContentId`, `UserId` newtypes.
- DB: Postgres 16, sqlx with compile-time-free runtime queries (`query_as`) so CI does not need `DATABASE_URL` at compile; migrations run on boot (`--migrate`) and in tests.
- Config: env vars (`KK_DATABASE_URL`, `KK_BIND`, `KK_BASE_URL`, `KK_SECRET`), no config files.
- Errors: one `ApiError` enum -> status + reference-shaped error JSON.
- Auth (M1): HTTP Basic `email:api-token`, tokens stored as sha256; bootstrap admin from env on first boot. OAuth, sessions: later.
- Observability: `tracing` JSON logs, `/healthz` and `/readyz`.
- Frontend: React 18, TS strict, Vite, `@tanstack/react-query` against a typed client; CSS via our tokens (`--kk-*`), no CSS-in-JS runtime.
- Deploy: container per binary; Helm chart with server Deployment, worker Deployment, Postgres as an external dependency, object storage config, ingress.

## CI (GitHub Actions) - required checks

| Job | Command |
|---|---|
| rust | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace` (Postgres service container) |
| web | `pnpm install --frozen-lockfile`, `pnpm -r typecheck`, `pnpm -r lint`, `pnpm -r test` (vitest), `pnpm build` |
| e2e | docker compose up, Playwright against the stack, upload screenshots and traces as artifacts |
| conformance | run the schema-mode conformance runner against the live stack |
| docs | `scripts/check-naming.sh`, inventory check, markdown link check, `cargo deny`-style license check later |
| commits | commitlint on PR titles/commits (Conventional Commits) |

## Commit convention

Conventional Commits: `type(scope): subject`, types `feat fix docs test refactor perf build ci chore`, scopes `core adf server web ui editor conformance deploy docs`. Subject imperative, <= 72 chars, body explains why. Breaking change: `!` or `BREAKING CHANGE:` footer. One logical change per commit.
