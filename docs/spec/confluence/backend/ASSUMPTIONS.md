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

## Attachments (M2)

- An attachment is a `content` row of type `attachment` (title = filename) plus `attachments` (container page, stable `fileId`) and `attachment_versions` (blob key, media type, size, comment per version). Only pages can contain attachments. Ids are exposed as `att<id>`; bare ids are accepted.
- Upload is `POST` or `PUT /wiki/rest/api/content/{id}/child/attachment` (`file`, `comment`, `minorEdit`). The same filename (case-insensitive) on the same page adds a version; `POST .../{attachmentId}/data` adds a version to a known attachment. Multipart is parsed by a small in-tree reader (no new dependency) and the whole request is buffered in memory.
- Size limit `KK_MAX_UPLOAD_BYTES`, default 25 MiB, over it is 413. No blocked-extension list. Filenames keep only the last path component; control characters and empty names are 400.
- Media type is sniffed from content (PNG, JPEG, GIF, PDF, ZIP, WebP, SVG magic numbers), then UTF-8 text by extension (`json`, `csv`, `md`, else `text/plain`), else `application/octet-stream`. The client media type is ignored.
- Blobs live in `KK_BLOB_DIR` (default `./blobs`) addressed by sha256 behind the `BlobStore` trait; a blob is deleted on purge only when no version references it. Versions of a live attachment never delete blobs.
- Download is `GET /wiki/download/attachments/{containerId}/{filename}?version=`, authenticated. Images (not SVG) and PDF are `inline`, everything else `attachment`, always `nosniff`. Responses carry `ETag` (the blob's sha256, strong), `Last-Modified` (the version's time, second precision) and `Accept-Ranges: bytes`; `If-None-Match` and `If-Modified-Since` answer 304 (the ETag wins when both are sent); one `bytes=` range answers 206, an unsatisfiable one 416, `If-Range` with a stale validator gets the whole body. Several ranges are served as a plain 200 (no multipart/byteranges). `[V]` that the reference behaves the same.
- `getAttachmentThumbnailById` has no image library behind it: an image (not SVG) answers 302 to the original download link, any other file 404; `width`/`height` are validated and ignored `[V]`. `getLabelAttachments` lists attachments carrying the label, with the same visibility filters as the other attachment lists; the v1 label endpoints accept `att` ids.
- `DELETE /attachments/{id}` trashes; `?purge=true` removes it at once, including from the current state. Trashed attachments appear only with `status=trashed`. Deleting a page removes its attachments with it.
- The v1 download endpoint redirects (302) to the download link instead of returning a JSON URI.
- `webuiLink` is `/spaces/<key>/pages/<pageId>?preview=%2F<pageId>%2Fatt<id>%2F<filename>`.
- Not built: blog post and custom content attachments, label attachments, `include-*` expansions, v1 `updateAttachmentProperties`, version delete and restore, thumbnails, antivirus hook, resolving ADF `media` filenames to attachments on render.

## Tasks (M2)

- Task rows are synced from `taskItem` nodes (by `localId`) whenever a page is created or published as `current`; rows of removed items are deleted, rows of trashed pages are hidden. The body stays the source of truth; drafts are not extracted.
- `body` is the item's inline content wrapped in one paragraph. `assignedTo` is the first `mention` whose id is an existing user id, `dueAt` the first `date` node. `createdBy` is the publishing user, `completedBy` and `completedAt` are set when a sync sees the item move to done and cleared when it moves back.
- Blank tasks (no text) are hidden unless `include-blank-tasks=true`. `blogpost-id` filters match nothing because blog posts are not built.
- `PUT /tasks/{id}` honours only `status`: it flips the item's state in the current body and publishes a minor-edit version with message "Task status changed". Other fields in the request are ignored.

## Permissions (M3)

All `[V]`: no reference capture exists for any of this; the public spec gives shapes, not rules.

- Subjects are users and groups. Group ids and account ids are numeric ids as strings. The built-in group `kk-users` has every user as an implicit member, cannot be deleted or edited, and carries the default grants of a new space. Anonymous access is not built (`anonymous` identifiers never match anything).
- Site administrators (`users.is_admin`) pass every space and content check. There is no admin-key flow. Mutating groups and global settings needs a site administrator.
- A new space grants its creator every space permission and `kk-users`: `read/space`, `create` page/blogpost/comment/attachment, `delete` comment/attachment. Spaces that existed before the permission tables get the same grants for `kk-users` and their author.
- `administer/space` implies every other space permission. `delete/space` is only recorded; space deletion is not built.
- No `read/space`: the space and everything in it is 404. Read allowed but an operation denied: 403 with code `FORBIDDEN`.
- Operation to permission: page `read` = view; `update`, `move` = `create/page` and the page's update restriction; `delete` = `delete/page` and the update restriction; `copy` = `create/page`; `archive` = `archive/page` and the update restriction; `restrict_content` = `restrict_content/space` or the right to update the page; comment `create` = `create/comment`; comment `update` = author; comment `delete` = author with `delete/comment`, or `administer/space`; attachment `create`/`update` = `create/attachment`, `delete` = `delete/attachment`. Space operations list every key/target pair the caller holds.
- Restrictions: per content row, per operation (`read`, `update`), a set of users and groups; no rows = unrestricted. View needs every ancestor-or-self read set (if any) to contain the caller (directly, or through a group) or the same level's update set. Update restrictions apply to the page itself only, not to descendants. Trashing keeps restrictions; purging removes them.
- Restricted content is excluded from lists, children, descendants, ancestors, search and watcher notifications; its comments and attachments follow the container.
- A restriction change that leaves a non-admin caller unable to view, or unable to update when they could before, is 400.
- `restrictionsHash` is the sha256 of the sorted restriction rows. The `expand` values `restrictions.user`, `restrictions.group` (also with the `read.`/`update.` prefix) fill the subject lists; `content` is accepted and ignored.
- Search index: `content_search` holds the plain text of the current version, its modified time and a stored `tsvector` (title plus text) with a GIN index; triggers on `content` and `content_versions` refresh it in the writing transaction (no indexing delay). Drafts are not indexed. CQL `text ~` is a stemmed (english) full-text match, `title ~` a case-insensitive substring match. Excerpt highlight markers are `@@@hl@@@` and `@@@endhl@@@`.
- CQL week starts on Monday; all relative dates are UTC.
- Notifications: a page create or publish enqueues a `notify.content` job in the same transaction; the worker inserts one `notifications` row per watcher (page watch, ancestor page watch for child-created, or space watch) who can view the content and is not the author. There is no public REST surface for reading notifications. Jobs are claimed with `FOR UPDATE SKIP LOCKED`, retried with a growing delay up to 5 attempts, then marked `failed`.
- Not built: v1 content CRUD, `/content/{id}/child`, `/content/{id}/history`, `GET /space`: they are absent from the pinned v1 spec so they cannot have a schema case (see report). Space roles, custom-content permissions, label watches, email delivery.

### Extensions and jobs (M3)
- v2 comment responses carry a top-level `authorId` (the original author). The pinned spec only has `version.authorId`, which changes on edit. This is a kaijukit extension and not `done`-able against the reference. `[V]`
- `getCurrentUser` returns the same shape as `/users-bulk` entries plus `type: known`. `getLabelPages` honours the same visibility filters as `getPages`.
- Jobs: table `jobs` (queued, done, failed), polled every 2 seconds by the server process; backoff is 5 s times 2^attempts. Notifications are stored in `notifications` and have no read API yet.
- v1 `getContentDescendants` returns `page`, `comment` and `attachment` arrays of 25. `depth` counts page levels; comments and attachments of the pages inside the depth are included.

## M4: page tree and version history (v1)

All `[V]`: the spec gives shapes only.

- `PUT /content/{id}/move/{position}/{target}`: `append` makes the page the last child of the target, `before`/`after` make it a sibling of the target (same parent, or root level). Positions of the new siblings are renumbered 0..n. The caller needs the same rights as updating the page, plus `create/page` and visibility of the target when the target is in another space. A page cannot move below itself or onto itself, and a space homepage cannot move (400). Moving across spaces takes the subtree, its comments and attachments along; a title clash in the target space is 400. Response `{"pageId"}`.
- `DELETE /content/{id}/version/{n}`: removes a historical version (page, blog post, comment or attachment, `att` ids accepted); the current version is 400, a missing one 404. Numbers are not reused or renumbered. A blob no remaining attachment version references is deleted. The reference rolls the change into the next version; we keep the later versions' bodies as stored.
- `POST /content/{id}/version` with `operationKey: restore`: appends a new version copying `params.versionNumber` (title only when `restoreTitle` is true; attachments always restore the old file name). Response is the new `Version` with `by`, `when`, `message`, `number`, `minorEdit` (no embedded `content`).
- v1 `updateAttachmentData` returns the single updated attachment (the spec says `Content`, not a list); v1 attachment download answers 302 as the spec lists.

## M4: blog posts, folders, whiteboards, databases, smart links, custom content (v2)

All are rows of the generic `content` table, so permissions, restrictions, trash, labels, properties, attachments, comments, versions and search reuse the page code.

- Blog posts: same write model as pages (versions, drafts, trash, purge). `[V]` blog `_links.webui` date path shape (`/spaces/KEY/blog/YYYY/MM/DD/id/title`), taken from the pinned spec examples, not from a live site.
- Blog operations report `targetType: blogpost`; create and delete on a blog post check the space's `blogpost` permission, not `page`.
- Folders, whiteboards, databases and smart links: create, get and delete only (the spec has no update). Create without `parentId` is root level; `parentId` may be any page or folder-like row in the same space. `[V]` that Confluence nests only some types under others; we allow any tree type under any other.
- Database entries, fields and views are empty lists; there is no entry store, so an entry id is always 404 with `ENTRY_NOT_FOUND`. `[V]` `DATABASE_NOT_FOUND` for a missing database is returned as the generic `NOT_FOUND` by the permission guard.
- Smart links need an `http` or `https` `embedUrl`; no fetching or unfurling happens.
- Custom content: `type` plus exactly one of `spaceId`, `pageId`, `blogPostId`, `customContentId`. A string body is stored verbatim and returned as `raw`; storage and ADF bodies convert as for pages. `[V]` that Confluence validates the type against an installed app; we accept any non-empty string. Create answers 201 as the spec says.
- `include-labels|properties|operations|likes|versions` on page and blog reads, and `include-properties|operations` elsewhere, add `{results, meta.hasMore, _links.self}` blocks capped at 50 items. `[V]` the cap and block item shapes; the spec only names the keys. `include-favorited-by-current-user-status` always reports false until favorites exist.

## M4b: v2 content permissions and restriction state (all kinds)

All `[V]`: shapes from the pinned spec, rules from our own v1 restriction model.

- `GET|POST|PUT|DELETE /{pages|blogposts|folders|whiteboards|databases}/{id}/permissions` and `GET|PUT .../restriction-state` share one implementation over the `content_restrictions` table. A kind's id on another kind's path is 404. Restrictions on any tree row govern everything below it (the oracle test mixes pages, folders, whiteboards and databases).
- POST adds principals. PUT replaces every row of each operation named in the body (operations not named are untouched). DELETE removes rows matching the optional `operation`, `principal-type`, `principal-id` filters (none = all) and answers 204. One entry per operation; unknown principals are 400.
- `allow-self-lockout=true` skips the check that rejects a change taking the caller's own view or update access away.
- `PUT restriction-state`: `OPEN` clears every row; `EDIT_RESTRICTED` leaves the caller alone on `update`; `VIEW_RESTRICTED` leaves the caller alone on `read` and `update`. The caller needs the right to restrict content.

## M4b: classification levels, admin key, data policies

All `[V]`.

- Migration 0011 seeds three levels (Public default, Internal, Confidential); there is no write API for the list (the spec has none). A content row reports its own level, else its space default, else the site default. `status` in GET/PUT/reset bodies and queries is validated by shape only: one level per row, no draft/current split. Unknown level id is 400; a row of another kind is 404. Applies to pages, blog posts, whiteboards and databases (the kinds the spec lists).
- `ClassificationLevel.status` is declared as both `object` and a string enum in the pinned spec; `allowed-diffs.json` skips the `type` complaint for it.
- Admin key: site administrators only (403 otherwise). POST stores an expiry of 1 to 60 minutes (default 10) for the caller; GET/DELETE are 404 without an active key. It grants nothing extra because administrators already pass every check.
- Data policies are not built: metadata says `anyContentBlocked: false`, the space list is empty.
- Not built: redactions (`postRedactPage`, `postRedactBlog`), space roles and role assignments, space permission transition, `convertContentIdsToContentTypes`, `inviteByEmail`.

## M4c: kaijukit extensions (not in the pinned specs)

Everything under `/wiki/rest/kk/v1` is ours: the pinned specs have no such operation, so cases for it set `"extension": true` (schema mode checks the status only) and no parity row exists. Never `done`-able against the reference.

- Notifications (read side of the M3 `notifications` table). All calls act on the caller's own rows.
  - `GET /notifications?start&limit&unread=true`: v1-style page (`results,start,limit,size,_links`), newest first, default limit 25. Item: `{id, kind (content.created|content.updated), read, createdAt, actor {accountId, displayName} or null, content {id, type, title, spaceKey, _links.webui}}`. Rows whose content is trashed or no longer visible to the caller are left out of the page (so a page can be short).
  - `GET /notifications/unread-count` -> `{count}`: unread rows on non-trashed content. It does not re-check restrictions, so it can exceed what the list shows.
  - `POST /notifications/{id}/read` -> 204; another user's or an unknown id is 404. `POST /notifications/read-all` -> 204.
- Users: `POST /users` `{email, displayName?, admin?}` (site administrators only, else 403) creates a user and an API token, returned once as `token` with `accountId`, `email`, `displayName`, `admin` (201). The email must have one `@` and a dotted domain, unique case-insensitively (400). Generated tokens are `kk_` plus 64 hex characters from the database's `gen_random_uuid()`; only the sha256 is stored. This is how a second, non-admin user is made without SQL.

## M4c: content templates (v1)

All `[V]`: shapes from the pinned v1 spec; rules ours. The spec has no "create page from template" operation, so none is built: a client reads the template with `templateExpand=body` and sends the body in an ordinary create-page call.

- Table `templates` (migration 0012): global (no space) or per space; body stored as ADF, accepted as `storage` or `atlas_doc_format`, returned as `storage` only with `templateExpand=body`. `templateId` is a numeric id string, `templateType` is always `page` (anything else is 400), `editorVersion` is `v2`.
- Global templates are created, updated and deleted by site administrators; space templates need `administer/space`. Reads: any user for global, `read/space` for space templates (else 404).
- `GET /template/page` without `spaceKey` lists the global templates only, with `spaceKey` that space's, ordered by name. `GET /template/blueprint` is always empty (no blueprints). `originalTemplate` and `referencingBlueprint` never appear.
- Update replaces name, description, labels and body; `space.key` must match the stored space (400). Labels are stored as names (prefix reported as `global`, `id` empty). Description over 255 characters is 400 (the update schema says 100; we use the create limit for both).
- Not built: template labels as real label rows, template versions, instantiating a blueprint.

## M4c: v1 user and group lookups

All `[V]`.

- `GET /user?accountId=` returns the long user shape (avatar `isDefault`, `isGuest=false`, `operations=[]`, `personalSpace=null`, `_expandable`); unknown or non-numeric ids are 404. `GET /user/bulk?accountId=` takes up to 100 ids (repeated or comma lists), skips unknown ones, answers 400 for none or too many. `expand` is accepted and ignored. `GET /user/anonymous` is a static record (anonymous access does not exist).
- `GET /user/email` and `/user/email/bulk` answer any authenticated caller (the reference restricts them to apps; we have no app identity yet, and `users-bulk` already returns emails). Bulk returns a bare array and skips unknown ids; the single form is 404 for an unknown id.
- `GET /user/memberof` lists the user's groups plus the built-in `kk-users`, ordered by name. `GET /group/picker?query=` is a case-insensitive substring match on the group name; `shouldReturnTotalSize` adds `totalSize`.
