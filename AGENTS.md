# AGENTS.md — rules for humans and AI agents

Last reviewed: 2026-11-24

1. **Claim work first.** Comment on the GitHub issue (or open one) before starting. One PR per issue. Never work an issue someone else has claimed.
2. **Clean room.** Read [docs/clean-room.md](docs/clean-room.md) before touching any spec. If unsure whether a source is allowed, it is not.
3. **Spec before code.** No feature code without a filled-in entry in `docs/parity/` (template in checklist.md) and a conformance case in `conformance/`.
4. **Naming.** Follow [docs/naming-policy.md](docs/naming-policy.md). `scripts/check-naming.sh` must pass.
5. **Ask before irreversible steps:** force-push, history rewrite, deleting branches/data, dropping DB migrations, publishing releases, changing the license, adding a dependency.
6. **Small, tested PRs.** Every parity line you change updates its status and links its tests. Don't mark `done` without a passing conformance case.
7. **Don't invent parity.** If the reference behavior wasn't observed or documented, mark it `not started` and say so.
8. **Core in Rust.** Domain model, query parser, workflow engine and permission checks live in core crates that build native and to WASM. No business logic in the server or UI layers.
9. **Confluence parity is a hard requirement.** See [docs/parity/confluence-parity.md](docs/parity/confluence-parity.md). Run `python3 -I scripts/confluence_inventory.py check <v1.json> <v2.json>` (spec URLs and hashes are in the script) before any PR that touches Confluence rows.
10. **`[V]` markers.** In `docs/spec/`, `[V]` means unverified. Never implement a `[V]` behavior as fact: capture it from the reference first, then remove the marker in the same PR. Regenerate `docs/parity/confluence-api/` with `scripts/confluence_api_spec.py` when the pinned spec changes; do not hand-edit it.
