# Conformance

Harness design: [docs/arch/conformance-harness.md](../docs/arch/conformance-harness.md). Runner in `runner/`, cases in `cases/<id>/case.json`.

```sh
pnpm install
pnpm --filter conformance-runner run check --base http://localhost:8080 --mode schema [--case id] [--spec v2.json]
pnpm --filter conformance-runner run check:inventory
pnpm --filter conformance-runner test
```

- Dev stack auth defaults to `admin@kaijukit.local:kk-dev-token`; override with `KK_AUTH=user:token`.
- Pinned spec URLs and sha256 live in `spec-pins.json`; without `--spec` the runner downloads to `.cache/` (gitignored) and verifies the hash.
- Setup steps chain responses: `{"as":"page1",...}` makes `{{page1.id}}` available later; `{{run}}` is a random suffix per run.
- `record` needs `REF_AUTH=user:token` for a reference instance you are licensed to use; it rewrites `expect` with normalized output and never stores credentials.
- Schema mode needs a live server; there is no teardown, so use a disposable dev database.
