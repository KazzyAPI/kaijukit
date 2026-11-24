# 04 Permissions

Last reviewed: 2026-11-24

Structure: [v2 Content Permissions](../../../parity/confluence-api/v2/content-permissions.md), [Space Permissions](../../../parity/confluence-api/v2/space-permissions.md), [Space Roles](../../../parity/confluence-api/v2/space-roles.md), [v1 Content restrictions](../../../parity/confluence-api/v1/content-restrictions.md), [v1 Space permissions](../../../parity/confluence-api/v1/space-permissions.md).

## Layers (evaluated in this order)

1. **Authentication** (anonymous, user, app, admin key) and **OAuth scopes** (token must hold the operation's scope or it is a 401/403 `[V]` which).
2. **Global permissions** (site level): Can use (browse site), Create space(s), Create personal space, Browse users/groups, Confluence administrator, System administrator, Anonymous use (read-only for unauthenticated). Granted to users and groups. Site admin bypasses space permissions only via the admin key flow (admin key must be enabled and is time-limited) `[V]`.
3. **Space permissions**: operation = (key, target). Keys/targets: `read/space`, `create/page`, `create/blogpost`, `create/comment`, `create/attachment`, `delete/page`, `delete/blogpost`, `delete/comment`, `delete/attachment`, `delete/space`, `export/space`, `administer/space`, `archive/page`, `restrict_content/space`, plus role-based permissions in the roles model `[V]`. Subjects: user, group, anonymous. Without `read/space` the space does not exist for the caller (404, not 403).
4. **Content restrictions**: per page/blog post, two operations: `read` (view) and `update` (edit); subjects users and groups. Empty restriction = no restriction. An `update` restriction implies the subjects can also view.
5. **Content-type rules**: comments editable only by their author (plus admins for delete); attachments follow their container; whiteboards/databases/folders follow the hierarchy.

## Evaluation

```
can(user, op, content):
  if !authenticated and !globalAnonymousAllowed -> deny
  if !space.hasPermission(user, read/space) -> not found
  if any ancestor-or-self has read restriction and user is not in that restriction set (direct or via group) -> not found
  if op needs edit/delete and (self has update restriction and user not in it) -> deny
  if !space.hasPermission(user, op.spaceKey) -> deny
  allow
```

- View restrictions are inherited from ancestors (cumulative: user must satisfy every ancestor's read restriction). Edit restriction inheritance: `[V]` confirm whether an ancestor's update restriction blocks editing of descendants; implement as reference behaves and record the capture.
- A restriction subject set that excludes its own creator is not allowed: creator is kept or the API rejects `[V]`.
- Restricted content is excluded from lists, search, children/descendants/ancestors, macro output, notifications, and counts. Parent chain with hidden middle pages renders hidden ancestors as absent in breadcrumbs `[V]`.
- `operations` array on a content/space response lists what the caller may do (`read, update, delete, copy, move, create_comment, create_attachment, restrict_content, archive, export, ...`) `[V]` exact enum from schema `OperationKey`/`PermittedOperationType`.
- Bulk restrictions API: add/remove subjects, get by operation, get for a user/group, and "inherited" computations; see the v1 restrictions inventory.

## Space roles (newer model) `[V]`

- v2 Space Roles: role id, principal type, assignments per space, role modes; assignment replaces the legacy matrix when the site is migrated. Support both models selectable per site, with the legacy matrix as default for API compatibility.

## Anonymous and public links

- Anonymous access: global switch + per-space `read/space` for anonymous. Public links on content ("share publicly") `[V]` exist in the UI; the API effect is a link token that bypasses restrictions for view of that one page.

## Data policies, classification, redaction

- Classification levels attach to spaces/content; enforcement actions defined by data policy (block export, block public links) `[V]`. Redactions mask ranges from view/export for non-privileged users.

## Admin key

- v2 Admin Key: enable (duration), get, disable. While enabled, the calling admin passes all space/content permission checks; each use is audited.

## Audit

- Permission changes, restriction changes, admin key use, and export are audit events with `changedValues`.
