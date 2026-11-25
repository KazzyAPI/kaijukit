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
- A PUT with `status=current` appends a version and requires `version.number == current + 1` (409 otherwise). Title-only update also appends a version, without a number check.
- Drafts: a never-published page is an ordinary row with status `draft` (PUT keeps appending versions, as before). A published (`current`/`archived`) page can also have one draft in `content_drafts` (title, body, author, updated_at). PUT or PUT `/title` with `status=draft` on a published page upserts that draft: no new version, no `version.number` check (autosave), response is the draft with `status=draft` and `version.number` still the published number, so the client publishes with number + 1. Draft title uniqueness is only enforced on publish. `GET /pages/{id}?get-draft=true` returns the draft (status `draft`, draft author and updated time as version author/time) when present, else the current page; it is ignored with `version=N`. Plain GET, lists, children and versions never show the draft. Publishing (PUT `status=current`) uses the PUT body, not the stored draft body, and deletes the draft. `DELETE ?draft=true` on a published page deletes only the draft (404 if none). Trashing keeps the draft; purging removes it.
- Updating a trashed page with `status=current` restores it and every row trashed by the same delete (`trashed_with`). `parentId`/`spaceId`/`ownerId` in PUT bodies are ignored (no move yet).
- DELETE: moves the page and its live descendants to `trashed`; `purge=true` requires `trashed` (400 otherwise) and hard-deletes the whole trashed subtree (no `deleted` status row); `draft=true` hard-deletes a `draft` page (404 otherwise; 400 if it has children). Deleting a space homepage is 400. DELETE of an already trashed page is 404.
- GET page defaults to status current/draft/archived; `status=trashed` is needed to read a trashed page. `version=N` returns that version's title and body with the page's current status.
- Page create returns the body in the representation that was submitted; update likewise. PUT/POST accept `storage` and `atlas_doc_format` only (`wiki` is 400).
- `view`, `export_view`, `anonymous_export_view`, `styled_view`, `editor` return the storage XML until macro rendering exists.
- Space create: key defaults to the first 10 alphanumerics of the name, uppercased; keys are unique case-insensitively (400). Type is always `global`. The homepage is a page titled with the space name.
- Descendants and children list only `current`/`archived` content; ancestors are root first and not paged (`limit` only). Descendants are ordered by depth, position, id (not depth-first).

## Users
- Users have `display_name`, `email`, `account_type` (default `atlassian`). The bootstrap admin is named `Admin` (an existing admin whose name still equals its email is renamed on start). `accountId` is the numeric user id as a string, matching `authorId`.
- `POST /users-bulk`: 1 to 100 ids else 400; non-numeric or unknown ids are skipped; results ordered by id; fields `accountId, displayName, publicName (= displayName), email, accountType, accountStatus=active, isExternalCollaborator=false`. No `profilePicture`, `timeZone`, `personalSpaceId`.
- `POST /user/access/check-access-by-email`: an email with no `@` is invalid; a valid email with no user row is without access. `invite-by-email` is not built (needs mail delivery).

## Body conversion (kk-adf)
- Nodes are generic (`type`, `attrs`, `content`, `marks`, `text`); no per-node validation yet.
- Unknown storage elements become `extension`/`inlineExtension` (`extensionType` `kk.storage`, `parameters.xml` holds the verbatim XML). ADF nodes or marks with no storage form are emitted as `<kk:adf-node>` with JSON in CDATA and restored on read; this element is ours, not reference storage.
- `colgroup`, ordered-list `start`, code macro parameters other than `language` (those macros stay raw extensions) and table attributes other than colspan/rowspan are not mapped yet. Round trip is semantic, not byte-identical (adjacent runs with a shared mark may be split).

## Body conversion (M2 additions, kk-adf)
- A converter claims a construct only when it can reproduce it exactly (attributes it does not map, extra marks, unknown colours and so on); everything else keeps the `<kk:adf-node>` fallback or the verbatim `kk.storage` extension, so no input is ever lost.
- Macros: `ac:macro-id` is the ADF `localId` (panel, expand, status, extensions) or `uniqueId` (codeBlock); `ac:schema-version` is kept only for generic macros (`parameters.macroMetadata.schemaVersion.value`). Panels: `info|note|warning` and `tip` (= `success`) macros; `error` is a `panel` macro with a `panelType` parameter (ours); `custom` panels use the fallback. `expand` has a `title` parameter. `nestedExpand` is emitted as the same `expand` macro and read back as `nestedExpand` only inside table cells.
- Generic macros (anything else with text-only parameters and at most one body) become `extension` (no body or plain-text body in `attrs.text`), `bodiedExtension` (rich-text body) or `inlineExtension` with `extensionType` **`kk.macro`**, `extensionKey` = macro name and `parameters.macroParams.<name>.value`. The spec in 09 names the reference's own `extensionType` string; the repo naming policy bans that string in code, so ours differs `[V]`. Duplicate parameter names or parameters with element children stay verbatim `kk.storage`.
- `status`: `title` and `colour` (Grey/Red/Yellow/Green/Blue/Purple = neutral/red/yellow/green/blue/purple) and `subtle=true` = `style: subtle`. `date` is `<time datetime="YYYY-MM-DD"/>` with a UTC-midnight millisecond `timestamp`; a timestamp that is not midnight uses the fallback. `mention` is `ac:link` + `ri:user` with the display text in `ac:plain-text-link-body`. `emoji.shortName` is `ac:emoji-shortname` (`ac:name` is derived without the colons), `id`/`text` map to `ac:emoji-id`/`ac:emoji-fallback`.
- Tasks: `taskItem.localId` = `ac:task-uuid`, `state` TODO/DONE = incomplete/complete, `ac:task-id` is a per-document counter written on output and ignored on input. `taskList.localId` has no storage home in the reference; we write it as `kk:local-id` on `ac:task-list`. Missing `localId`s are not synthesized.
- Decisions use the `ac:adf-extension` / `ac:adf-node type="decision-list"` shape with `ac:adf-attribute` children `[V]` (written from memory of the reference, not captured).
- Media: `<p>` holding only `ac:image` is a `mediaSingle` (layout = `ac:layout`, width = `ac:width`, media `alt`, `width`, `height` = `ac:alt`, `ac:original-width`, `ac:original-height`, caption = `ac:caption`). The attachment filename is `media.attrs.__fileName` (`ri:filename`); `id` and `collection` ride on our `kk:media-id` / `kk:collection` attributes of `ri:attachment`. Reading reference storage therefore yields media without `id`; no filename-to-`fileId` lookup yet, and images with `ac:align` or other attributes stay verbatim. `mediaGroup` is one `view-file` macro with a `name` parameter per file (more than one is ours). `mediaInline` is not mapped.
- Layout widths come from the section type (`two_left_sidebar` = 66.66/33.33 and so on); widths that match no type use the fallback. Consecutive sections are written as separate `ac:layout` wrappers.
- Smart links: `<a data-card-appearance="inline|block">`; inline text is ignored on read. Marks: `span style` colour/background-colour, `sub`/`sup`, `u`, `s`, `ac:inline-comment-marker` (annotation `inlineComment`), paragraph/heading `style` `text-align` (`center`, `right` = `end`) and `margin-left` in 30px steps (= `indentation.level`). Mark order is not normalized: a paragraph with `indentation` before `alignment` reads back as alignment first.
- Marks or attributes on a block the converters do not map (for example `breakout`) now use the fallback instead of being dropped; link marks with attributes besides `href` likewise.

## Not built (still in the generated spec)
- Space query params `labels`, `favorited-by`, `include-*`, `description-format=view` rendering; page `subtype`, `include-*`, `private`, `embedded`; folder/whiteboard/database/embed hierarchy endpoints; `icon`, `currentActiveAlias`; audit rows; TLS for the database connection.

## Comments (M2)

- A comment is a `content` row of type `comment` plus a `comments` row (kind, container, parent, anchor, resolution), so versions, properties and likes reuse the generic tables.
- Comment titles are `Re: <container title>`. Replies inherit the container of their parent; a reply to an inline comment is an inline comment without its own anchor.
- Comment updates require `version.number` equal to current plus one, else 409. Delete is a hard delete and cascades to replies.
- Inline create anchors the selection inside a single text node of the page's current version body and does not create a page version. Selections spanning several text nodes are rejected as not found. Errors are 400 with the reference wording.
- The marker ref is a random UUID. Deleting an inline comment removes its annotation mark from the current body.
- Publishing a page body that lacks a marker turns open or reopened inline comments into `dangling`; dangling comments cannot be resolved or reopened.
- `resolved: true` sets `resolved`; `false` on a resolved comment sets `reopened`. `resolutionLastModifierId` and `resolutionLastModifiedAt` appear once a comment was resolved or reopened.
- Attachment comments are exposed with the `att` id prefix; both prefixed and bare ids are accepted. Blog post and custom-content comments return 400.
- Comment `_links.webui` is `/spaces/<key>/pages/<pageId>/<Title+With+Plus>?focusedCommentId=<id>`.
- `KK_PORT` in the compose file only overrides the published host port and base URL for side-by-side stacks.

## Labels, properties, likes, operations (M2)

- Label names are trimmed, lowercased, 1 to 255 characters, no whitespace. Content labels default to prefix `global`, space labels to `team`. `GET` label lists apply no default prefix filter, including the space lists where the reference defaults to `my, team`.
- Label writes use the v1 endpoints (`POST/DELETE /wiki/rest/api/content/{id}/label`, `DELETE .../label/{name}`, `POST/DELETE /wiki/rest/api/space/{key}/label`) because v2 has only reads. Adding an existing label is a no-op that returns the full list.
- Not built: `getLabelPages`, `getLabelAttachments`, blog post and custom content labels, `include-labels` and `include-properties` expansions on page and comment reads.
- Property keys are 1 to 255 characters; serialized values are capped at 32 KiB (413 above). Duplicate key is 409. Update needs `version.number` equal to current plus one (409 otherwise) and an unchanged key (400). Content property create returns 200 and space property create 201, as the spec lists.
- Properties exist for pages, attachments, comments and spaces only.
- App properties are global per instance, accept any authenticated user, store the request body as the value; PUT returns 201 on create, 200 on update. The 403 case is not produced.
- Likes have no v2 write. `PUT/DELETE /wiki/rest/api/content/{id}/like` is a KaijuKit extension, idempotent, on pages and comments. `limit=0` on like users is rejected (minimum 1 enforced).
- Operations lists are static (every authenticated user holds every operation) until permissions land in M3. The operation and target pairs follow the permission model in 04.
