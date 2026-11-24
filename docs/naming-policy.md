# Naming policy

Last reviewed: 2026-11-24

Our UI, API output, error messages, default data and package names must not use third-party product names or marks.
Banned terms (case-insensitive): see `scripts/banned-terms.txt`.

Allowed only in: `docs/`, `conformance/`, `README.md` and `AGENTS.md`, where they're used to state compatibility factually ("compatible with ...").

Enforced by `scripts/check-naming.sh` (run in CI once CI exists).
