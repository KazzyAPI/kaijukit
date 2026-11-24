# Conformance harness

Last reviewed: 2026-11-24

Purpose: make every parity claim measurable. Lives in `conformance/runner` (TypeScript, Node).

## Case format (`conformance/cases/<id>/case.json`)

```json
{ "id": "v2-spaces-list", "ops": ["getSpaces"], "setup": [], "request": {"method":"GET","path":"/wiki/api/v2/spaces","query":{"limit":"2"},"headers":{},"body":null},
  "expect": {"status":200,"body":{...},"headers":{"content-type":"application/json"}},
  "normalize": {"ignore":["$.results[*].createdAt"],"placeholders":{"id":"$.results[*].id"}} }
```

`ops` lists operationIds the case covers; `docs/parity/confluence-rest-v*.md` Case column points to the case id. `setup` is an ordered list of requests whose responses feed variables (`{{page1.id}}`) into later requests so cases are self-contained.

## Modes

| Mode | Needs reference | Passes when |
|---|---|---|
| `schema` | no | response status is documented for the operation and the body validates against the pinned OpenAPI response schema (OpenAPI 3.0 -> JSON Schema, `nullable` handled) |
| `diff` | recorded fixtures only | normalized response equals the recorded reference response |
| `record` | yes | writes `expect` from a live reference instance (credentials via env, scrubbed) |

A row may be `done` only through `diff` (true parity) or, until a reference exists, `partial (schema)`. The inventory Status column uses `partial` for schema-only coverage.

## Normalization
Replace ids/uuids/accountIds with stable placeholders by first occurrence order, drop volatile fields (timestamps, `_links.base`, version `createdAt`), sort unordered arrays when the spec says unordered, ignore header ordering.

## Runner CLI
`pnpm --filter conformance-runner run check --base http://localhost:8080 --mode schema [--case id]` exits non-zero on any failure and prints a per-case table. `check:inventory` verifies every `Case` cell in the inventories names an existing case and every case's `ops` exist in the pinned spec.

## Reference safety
Recording happens only against an instance the maintainers own and are licensed to use; credentials never committed; cases scrubbed of personal data (see clean-room policy).
