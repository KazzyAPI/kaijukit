# Backend assumptions (M0/M1)

Last reviewed: 2026-11-24

Choices made where the reference gives no visible signal or is marked `[V]`. Each needs a captured reference case before it is treated as parity.

## Errors and auth
- Error body is `{"errors":[{"status","code","title","detail"}]}` with codes `INVALID_REQUEST_PARAMETER`, `UNAUTHORIZED`, `NOT_FOUND`, `CONFLICT`, `REQUEST_ENTITY_TOO_LARGE`, `INTERNAL_SERVER_ERROR`; `detail` text is ours. 401 adds `WWW-Authenticate: Basic`.
- Email matching for Basic auth is case-insensitive; tokens are compared by sha256. `authorId`/`ownerId`/version `authorId` are the numeric user id as a string.
- No permission checks yet: any authenticated user can read and write everything (M3).

## Pagination
- Cursors are base64url of a row offset (`o:<n>`), opaque to clients, stable for repeated reads of unchanged data, and may skip or repeat rows under concurrent writes. Bad cursor, `limit` outside 1..250: 400. `_links.next` is a relative `/wiki/api/v2/...` URL preserving the other query params; `Link: <url>; rel="next"` mirrors it. `_links.base` is `KK_BASE_URL` + `/wiki`.
- Unspecified sorts: pages and spaces by `id` ascending; versions by number ascending; children by position.

## Content model
- Title uniqueness is case-insensitive per space among pages with status current/draft/archived and a non-empty title; violation is 400. Restoring a trashed page whose title was taken is also 400.
- Create without `parentId` parents under the space homepage; `root-level=true` gives a null parent; `parentId` must be a live content in the same space (else 404). Position is append-only (max + 1).
- Every PUT (including draft updates) appends a version and requires `version.number == current + 1` (409 otherwise). Drafts are ordinary rows with status `draft`; there is no separate draft-of-published row. Title-only update also appends a version, without a number check.
- `current -> draft` is 400. Updating a trashed page with `status=current` restores it and every row trashed by the same delete (`trashed_with`). `parentId`/`spaceId`/`ownerId` in PUT bodies are ignored (no move yet).
- DELETE: moves the page and its live descendants to `trashed`; `purge=true` requires `trashed` (400 otherwise) and hard-deletes the whole trashed subtree (no `deleted` status row); `draft=true` hard-deletes a `draft` page (404 otherwise; 400 if it has children). Deleting a space homepage is 400. DELETE of an already trashed page is 404.
- GET page defaults to status current/draft/archived; `status=trashed` is needed to read a trashed page. `version=N` returns that version's title and body with the page's current status.
- Page create returns the body in the representation that was submitted; update likewise. PUT/POST accept `storage` and `atlas_doc_format` only (`wiki` is 400).
- `view`, `export_view`, `anonymous_export_view`, `styled_view`, `editor` return the storage XML until macro rendering exists.
- Space create: key defaults to the first 10 alphanumerics of the name, uppercased; keys are unique case-insensitively (400). Type is always `global`. The homepage is a page titled with the space name.
- Descendants and children list only `current`/`archived` content; ancestors are root first and not paged (`limit` only). Descendants are ordered by depth, position, id (not depth-first).

## Body conversion (kk-adf)
- Nodes are generic (`type`, `attrs`, `content`, `marks`, `text`); no per-node validation yet.
- Unknown storage elements become `extension`/`inlineExtension` (`extensionType` `kk.storage`, `parameters.xml` holds the verbatim XML). ADF nodes or marks with no storage form are emitted as `<kk:adf-node>` with JSON in CDATA and restored on read; this element is ours, not reference storage.
- `colgroup`, ordered-list `start`, code macro parameters other than `language` (those macros stay raw extensions) and table attributes other than colspan/rowspan are not mapped yet. Round trip is semantic, not byte-identical (adjacent runs with a shared mark may be split).

## Not built (still in the generated spec)
- Space query params `labels`, `favorited-by`, `include-*`, `description-format=view` rendering; page `subtype`, `include-*`, `private`, `embedded`; folder/whiteboard/database/embed hierarchy endpoints; `icon`, `currentActiveAlias`; audit rows; TLS for the database connection.
