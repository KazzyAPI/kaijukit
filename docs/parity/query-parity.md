# Query language and search

Last reviewed: 2026-11-25

Entry template: [checklist.md](checklist.md). Grammar source: [05-search-cql.md](../spec/confluence/backend/05-search-cql.md) (written from public docs; `[V]` marks unobserved behavior). Parser lives in `crates/kk-core/src/cql`.

### CQL parser
- Behavior: parses `expr [ORDER BY field [ASC|DESC], ...]`; `AND` binds tighter than `OR`; `NOT`, parentheses, `field op value`, `field [NOT] IN (...)`; keywords case-insensitive; strings quoted with `"` or `'` and `\` escapes; function calls with string arguments.
- Source: 05-search-cql.md grammar section.
- API shape & edge cases: syntax errors carry the character offset; unknown fields, operators not valid for a field, bad dates and unknown functions are separate error kinds. The exact error text of the reference is not captured, ours is free text `[V]`.
- Status: partial
- Tests: kk-core `cql` unit tests, conformance/v1-search-cql-invalid

### CQL fields and functions
- Behavior: supported fields are `type` (page, comment, attachment, blogpost; `space` and `user` answer 400 "not supported"), `space`/`space.key`, `title`, `text`, `label`, `creator`, `created`, `lastModified`, `id`/`content`, `ancestor`, `parent`, `status`. `currentUser()`, `startOf/endOf Day|Week|Month|Year` (optional offset like `"-1w"`), `now()` evaluate in UTC. Fields from the spec list that need unbuilt features (`favourite`, `watcher`, `mention`, `contributor`, `macro`, `instanceType`) answer 400 "not supported".
- Source: 05-search-cql.md fields and functions sections.
- API shape & edge cases: week starts Monday `[V]`; `text ~` uses stemmed full-text matching, `title ~` is a case-insensitive substring match `[V]`.
- Status: partial
- Tests: kk-core `cql` unit tests, `search_*` in crates/kk-server/tests/api.rs

### Search endpoints
- Behavior: `GET /wiki/rest/api/search` returns search results (content, space or user entities) for a CQL query; `GET /wiki/rest/api/content/search` returns plain content for a CQL query; `GET /wiki/rest/api/search/user` searches users by CQL. Results never include content the caller cannot see, and `totalSize` counts only visible results.
- Source: swagger.v3.json `searchByCQL`, `searchContentByCQL`, `searchUser`.
- API shape & edge cases: `start`/`limit` paging, `_links.next`; `excerpt` strategies other than `none` and `highlight` behave as `highlight`; `cursor`, `next`, `prev`, `cqlcontext`, `includeArchivedSpaces`, `sitePermissionTypeFilter` are accepted and ignored except `includeArchivedSpaces` `[V]`. Ranking is title match first, then recency; not compared to the reference. There is no separate index: text is extracted from the current version at query time (sequential scan, fine for small sites, needs a stored tsvector and GIN index before large ones). `/search/user` supports `user.fullname` and `user.accountid` only. There is no v2 search operation in the pinned v2 spec.
- Status: partial
- Tests: conformance/v1-search, v1-search-space, v1-content-search, v1-search-user, v1-search-cql-invalid, `search_*` in crates/kk-server/tests/api.rs
