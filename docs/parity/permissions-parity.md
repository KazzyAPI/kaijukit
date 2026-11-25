# Permissions and schemes

Last reviewed: 2026-11-25

Entry template: [checklist.md](checklist.md). Design notes and the choices made where the reference is silent: [ASSUMPTIONS.md](../spec/confluence/backend/ASSUMPTIONS.md) section "Permissions (M3)". Every entry stays `partial` until a diff-mode case exists.

### Space permissions model
- Behavior: a space grants (operation key, target) pairs to users and groups. Without `read/space` the space does not exist for the caller (404). `administer/space` implies every space permission. Site administrators (`is_admin`) pass every check (no admin-key flow yet).
- Source: <https://dac-static.atlassian.com/cloud/confluence/swagger.v3.json> `SpacePermissionRequest`; [04-permissions.md](../spec/confluence/backend/04-permissions.md).
- API shape & edge cases: valid pairs are `create` on page/blogpost/comment/attachment, `read/space`, `delete` on page/blogpost/comment/attachment/space, `export/space`, `administer/space`, `archive/page`, `restrict_content/space`. Anything else is 400. Anonymous subjects are not built.
- Status: partial
- Tests: `permissions_*` in crates/kk-server/tests/api.rs, kk-core `perm` unit tests, conformance/v1-space-permission-add, v1-space-permission-remove

### Content restrictions (view and edit)
- Behavior: a page may carry a `read` subject set and an `update` subject set (users and groups). An empty set means no restriction. A read restriction hides the page and its whole subtree from everyone outside the set (404, absent from lists, children, search). An `update` restriction limits editing, deleting and archiving of that page; subjects of `update` can also view. Update restrictions are not inherited by descendants `[V]`.
- Source: swagger.v3.json tag "Content restrictions" (ContentRestrictionArray, ContentRestrictionAddOrUpdateArray); [04-permissions.md](../spec/confluence/backend/04-permissions.md).
- API shape & edge cases: POST adds, PUT replaces per operation and subject type, DELETE clears. A change that would remove the caller's own ability to view (or edit, when they could) is rejected with 400 `[V]`.
- Status: partial
- Tests: `restrictions_*` in crates/kk-server/tests/api.rs, conformance/v1-restrictions-* (12 cases)

### Content permission check
- Behavior: `POST /content/{id}/permission/check` answers whether a user or group may read, update or delete the content, with the failing reason.
- Source: swagger.v3.json `ContentPermissionRequest`, `PermissionCheckResponse`.
- API shape & edge cases: `anonymous` identifiers answer `hasPermission: false`; error message text is ours `[V]`.
- Status: partial
- Tests: conformance/v1-content-permission-check

### Permitted operations (computed)
- Behavior: `operations` lists on pages, spaces, comments and attachments are computed from the caller's permissions and restrictions, not static.
- Source: v2 `getPageOperations`, `getSpaceOperations`, `getFooterCommentOperations`, `getInlineCommentOperations`, `getAttachmentOperations`.
- API shape & edge cases: mapping of operation to permission is in ASSUMPTIONS.md `[V]`.
- Status: partial
- Tests: v2-page-operations, v2-space-operations, v2-footer-comment-operations, v2-inline-comment-operations, v2-attachment-operations

### Groups
- Behavior: named groups with members; the built-in group `kk-users` contains every user and cannot be edited or deleted.
- Source: swagger.v3.json tag "Group".
- API shape & edge cases: names unique case-insensitively; mutations need a site administrator (403 otherwise). `searchGroups` (`/group/picker`) is not built.
- Status: partial
- Tests: conformance/v1-group-*

### Watches
- Behavior: a user can watch a page or a space; creating or editing a current page enqueues a `notify.content` job that records an in-app notification for each watcher (except the actor) who can still view the content. Jobs live in Postgres, are claimed with SKIP LOCKED, retry with exponential backoff and are marked failed after 5 attempts.
- Source: swagger.v3.json tag "Content watches" (user/watch/content, user/watch/space, notification/created, notification/child-created).
- API shape & edge cases: label watches, `getWatchersForSpace` and `getWatchesForSpace` are not built. There is no reference REST surface for reading notifications, so ours is internal only.
- Status: partial
- Tests: conformance/v1-watch-*, `watches_notify_through_the_job_queue` in crates/kk-server/tests/api.rs
