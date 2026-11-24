# 02 Content lifecycle

Last reviewed: 2026-11-24

Structure of the operations: [v2 Page](../../../parity/confluence-api/v2/page.md), [Blog Post](../../../parity/confluence-api/v2/blog-post.md), [Version](../../../parity/confluence-api/v2/version.md), [v1 Content](../../../parity/confluence-api/v1/content.md).

## States and transitions

```
(create status=draft) -> draft --publish--> current --edit--> current (version+1)
(create status=current) ------------------> current
current --trash--> trashed --restore--> current
trashed --purge (delete with purge=true, or retention job)--> deleted
current --archive--> archived --unarchive--> current        (archive is a space-level bulk op too)
```

- Create: default status is `current` (published). `status=draft` creates an unpublished draft; title may be empty for drafts, required for published (validation 400). Title must be unique among pages in the same space (400 on conflict `[V]` exact message). Blog posts: unique per space and posting date `[V]`.
- `parentId` omitted and `root-level=false`: parent is the space homepage. `root-level=true` with `parentId` is a 400. `private=true` creates a page only the author can see/edit (restrictions applied).
- Request body size limit 5 MB, otherwise 413.
- Update (PUT page): requires `id`, `status`, `title`, `body`, and `version.number` = current + 1; a stale number is a 409. `version.message` optional. Changing `status` current -> draft is not allowed `[V]`. Update may change `parentId`/`spaceId`? Not in v2 body; moves use the v1 move endpoint or `PUT /pages/{id}/move` equivalents as present in the inventory.
- Title-only update (`PUT /pages/{id}/title`) creates a version.
- Draft edits autosave to the draft row without a new version; publishing converts the draft to the next `current` version and records `version.message`/`minorEdit` `[V]`.
- Minor edit: suppresses watcher notifications; still a version.
- Delete (`DELETE /pages/{id}`): moves to `trashed`. With `purge=true` on an already trashed page: permanent delete. With `draft=true`: delete the draft only. Permission needed: delete page in space (and edit restriction pass).
- Restore from trash: via update with `status=current` `[V]`, or v1 content update. Restores descendants trashed with it.
- Move: to new parent and/or space; position semantic `append`/`before`/`after` `[V]`. Moving to a different space moves attachments, labels stay, restrictions are cleared/kept per reference `[V]`. Moving a page under its own descendant is rejected.
- Copy (v1): with or without attachments/permissions/labels/descendants; async via long-running task `[V]`.
- Reorder: sibling position update; also happens on move.
- Homepage: deleting the homepage is blocked; space homepage can be changed in settings.
- Hierarchy ops: children (direct, ordered by position), descendants (depth `limit`/`depth`), ancestors (root first `[V]`). Folders, whiteboards, databases and smart links appear in these results with their `type`.

## Versions

- Version number increments per published edit. `GET /pages/{id}/versions` lists (cursor paging, `sort`), `GET .../versions/{n}` returns one with body if requested.
- Diff is a UI concern computed from two version bodies (see frontend 03).
- Restore a version = create a new version whose body equals the old one (v1 content versions restore operation `[V]`), not a pointer move.
- Version delete (v1): removes a historical version, allowed to space admins `[V]`.
- Page "author" is the original creator; "last edited by" is `version.authorId`.

## Titles, slugs, links

- Web URL forms: `/wiki/spaces/{KEY}/pages/{id}/{Title-with-plus-or-dashes}` (`id` authoritative, title slug cosmetic). Also `/wiki/spaces/{KEY}/overview`, `/wiki/pages/viewpage.action?pageId={id}`, `/wiki/display/{KEY}/{Title}` (legacy) and short links `/wiki/x/{tinyId}` where tinyId is the base64url-style encoding of the numeric id `[V]` algorithm. All must resolve and redirect to the canonical URL. `_links.webui`, `tinyui`, `editui`, `self` are returned on content.
- Renaming a page updates inbound links stored as `ri:page` content-title references `[V]` (storage links by title are rewritten on rename).

## Concurrency

- Optimistic: version number check (409). With realtime editing on, API updates against a document with an active session merge or fail per [13](13-realtime.md).
