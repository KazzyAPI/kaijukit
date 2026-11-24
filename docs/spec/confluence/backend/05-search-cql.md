# 05 Search and CQL

Last reviewed: 2026-11-24

Structure: [v1 Search](../../../parity/confluence-api/v1/search.md). Endpoints include content search by CQL, site search, user search, and the (v1) "scan"/pagination variants listed in the inventory.

## Pipeline

1. Parse CQL -> AST (core Rust crate `cql`, native + WASM; the frontend uses the same parser for validation and autocomplete).
2. Authorize: add space-read and content-restriction filters server-side; the caller can never match hidden content.
3. Plan -> Postgres full-text (`tsvector` on title and flattened text, trigram on title for `~` fuzzy) plus relational filters. OpenSearch is a later adapter behind the same trait.
4. Rank, paginate (`start`/`limit`, `cursor` on the cursor variant), shape results.

## Grammar

```
query      := expr [ ORDER BY ordering ("," ordering)* ]
expr       := term ( (AND | OR) term )*          ; AND binds tighter than OR; parentheses group
term       := NOT term | "(" expr ")" | clause
clause     := field op value | field IN "(" value ("," value)* ")" | field NOT IN "(" ... ")"
op         := = | != | ~ | !~ | > | >= | < | <=
value      := string | number | function | relative-date
ordering   := field [ASC | DESC]
```

Operators by field type (`[V]` per field): `=`/`!=` exact; `~`/`!~` contains/fuzzy for text; `>` etc. for dates and numbers; `IN`/`NOT IN` lists. Keywords are case-insensitive. Strings quoted with `"` or `'`, `\` escapes.

## Fields (initial set `[V]` against reference)

`type`, `space`, `space.type`, `space.key`, `title`, `text`, `label`, `creator`, `contributor`, `mention`, `favourite`, `favouriteOf`, `watcher`, `created`, `lastModified`, `id`/`content`, `ancestor`, `parent`, `container`, `macro`, `instanceType`, `ancestor`, `title`, `status`? `[V]`. Content `type` values: `page`, `blogpost`, `comment`, `attachment`, `space`, `user`, `whiteboard`, `database`, `folder`, `embed`.

## Functions

`currentUser()`, `currentSpace()`? `[V]`, `startOfDay()`, `startOfWeek()`, `startOfMonth()`, `startOfYear()`, `endOfDay()`, `endOfWeek()`, `endOfMonth()`, `endOfYear()` with optional offsets like `"-1w"`, `"+2d"`; `now()` `[V]`. Date literals: `"YYYY-MM-DD"`, `"YYYY/MM/DD"`, `"YYYY-MM-DD HH:mm"`, relative `"-4w"`.

## Errors

- Invalid CQL: 400 with the reference's message shape (exact strings captured `[V]`), including the position of the error where the reference provides it. Unknown field, bad operator for field, unparsable date, unknown function are distinct cases.

## Results

- Result item: `{ content|space|user|..., title, excerpt, url, resultGlobalContainer, entityType, iconCssClass, lastModified, friendlyLastModified, timestamp, score? }`. Excerpt with match highlight markers (`@@@hl@@@` in v1 results `[V]`) and `includeArchivedSpaces`, `excerpt` strategy param.
- `totalSize`, `cqlQuery`, `searchDuration`, `_links.next`.
- Site search (UI search) accepts text + filters, converts to CQL internally. User search by name/email with privacy rules.
- Ranking: title match > body match, recency boost, favorites boost `[V]`; exact ranking cannot be matched byte-for-byte; the parity rule is: the same top-N set for the conformance queries (ordering within ties may differ), recorded in `conformance/search/`.
- Indexing latency: new content is searchable within N seconds; the reference value `[V]` is measured, and ours must be <= that. Index updates are enqueued in the same transaction as the write (outbox) and applied by a worker.
- Permission-trimmed counts: `totalSize` must not leak hidden-content counts.
- Search of attachment text (PDF/Office text extraction) `[V]` supported; extraction job in [11](11-events-jobs.md).

## Autocomplete / helpers

- Quick-nav search (recent + typeahead), mention search (users), space search, label search, link picker search, macro-browser pickers. All reuse the same engine with distinct endpoints (v1 `search/user`, `content/...` list endpoints) as in the inventory.
