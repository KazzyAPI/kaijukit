# 06 API conventions

Last reviewed: 2026-11-24

These apply to every operation unless the generated structure says otherwise.

- **Base paths:** `/wiki/api/v2`, `/wiki/rest/api`. Site root hosts the UI at `/wiki/...`.
- **IDs:** v2 returns ids as strings; path ids accept digits. v1 returns content ids as strings and space ids as numbers `[V]`. User ids are `accountId` strings.
- **Auth:** HTTP Basic (email + API token), OAuth 2.0 authorization code (3LO) with the scopes listed per operation, plus sessions for the UI (cookie + CSRF header `X-Atlassian-Token: no-check` semantics `[V]` for v1 form posts). Scope names are exactly those in `securitySchemes` ([_schemas](../../../parity/confluence-api/v2/_schemas.md)).
- **Pagination v2:** `limit` (default 25, max 250 for most, per-op max is in the structure), `cursor` opaque; next page in the `Link` response header (`rel="next"`) and `_links.next`. Cursors are opaque, stable for repeated reads, invalid cursor -> 400. `_links.base` present at top level.
- **Pagination v1:** `start`, `limit`, response `size`, `limit`, `start`, `_links.next/prev/base/context`. Max `limit` is operation specific (commonly 100 for expanded bodies, up to 200 `[V]`). Large `expand` reduces the cap.
- **Expansion v1:** `expand=a.b,c` over documented expandable properties; unknown expand ignored; `_expandable` map returned for unexpanded properties.
- **Sorting:** per-op `sort` enum, `-field` for descending.
- **Errors:** v2 returns `{ errors: [{ status, code, title, detail }] }` `[V]`; v1 returns `{ statusCode, data:{...}, message, reason }`. Status codes per the structure; 401 missing/invalid creds, 403 forbidden, 404 not found/hidden, 409 version conflict, 413 too large, 429 rate-limited. Error strings are captured, not paraphrased.
- **Rate limiting:** cost-based limits per app/user with `X-RateLimit-*`, `Retry-After` on 429 `[V]` headers; our limiter must emit compatible headers so clients back off identically. Defaults documented and configurable.
- **Caching/conditional:** ETag on GET where the reference provides it `[V]`.
- **Idempotency:** none implied; duplicate title create returns the conflict error.
- **Content negotiation:** JSON only; `Content-Type: application/json`; multipart for attachment upload; `Accept-Language` affects only error text.
- **Long-running tasks:** copy/move trees, export, import and space delete return `{ id, links }` -> poll `/longtask/{id}`. Statuses and percentage fields per v1 "Long-running task".
- **Webhooks and events:** see [11](11-events-jobs.md).
- **OpenAPI:** the server generates its own OpenAPI from Rust types; CI diffs it against the pinned spec for every operation (paths, params, required flags, enums, response codes). Differences must be zero or explicitly listed in an `allowed-diffs` file with a reason.
- **Versioning of our API:** we track the pinned snapshot only; additions beyond the reference live under `/wiki/ext/` and never alter reference shapes.
- **Compatibility toggles:** none that change reference-shaped output.
