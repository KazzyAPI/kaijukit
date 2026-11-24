# 08 Comments, labels, properties, likes, watches, states

Last reviewed: 2026-11-24

Structure: [v2 Comment](../../../parity/confluence-api/v2/comment.md), [Label](../../../parity/confluence-api/v2/label.md), [Content Properties](../../../parity/confluence-api/v2/content-properties.md), [Like](../../../parity/confluence-api/v2/like.md), [Task](../../../parity/confluence-api/v2/task.md), [Space Properties](../../../parity/confluence-api/v2/space-properties.md), [App Properties](../../../parity/confluence-api/v2/app-properties.md), [v1 Content watches](../../../parity/confluence-api/v1/content-watches.md), [v1 Content states](../../../parity/confluence-api/v1/content-states.md).

## Comments

- Two kinds: footer (page-level, threaded by `parentCommentId`) and inline (anchored to a text range of the page body).
- Create footer: needs `create/comment`, view access to the page. Body in `storage` or `atlas_doc_format`.
- Create inline: body plus `inlineCommentProperties { textSelection, textSelectionMatchCount, textSelectionMatchIndex }` which the server uses to place the `ac:inline-comment-marker` / ADF annotation on the page body, creating a **page version** `[V]` or an internal non-versioned annotation update. Selection not found -> 400.
- Resolve/reopen: inline comment `resolved` flag and `status` (`open`,`reopened`,`resolved`,`dangling`) `[V]`. Resolution records who/when. Reopening an orphaned (`dangling`) comment is not possible.
- Orphaning: if the anchored text is deleted by edit, the comment becomes `dangling`; marker removed but the comment remains accessible in the panel.
- Edit own comment; delete own (or admin). Deleting a parent with replies deletes the thread `[V]`.
- Reactions on comments (emoji) `[V]` if present in reference UI.
- Comments inherit view restrictions of their page; `mention` in a comment triggers a mention notification.
- Versions: comments have versions (`version.number`), list/get operations exist.

## Labels

- Label: `name` lowercase, no spaces, allowed chars restricted `[V]` regex, max length 255; prefix `global` (default), `my` (personal), `team`, `system`.
- Add/remove on pages, blog posts, attachments, custom content, spaces and templates. Adding requires edit permission on the content (space labels: admin). Duplicate add is a no-op returning the list.
- Label info endpoints: popular/related labels, content for a label (restriction-trimmed).

## Properties

- Content properties: `key` (<=255, `[a-zA-Z0-9_.-]`... `[V]`), `value` JSON (limit `[V]`), `version.number` incremented on update with optimistic check. Per content id, created by users/apps. Special keys (e.g. editor settings, `content-appearance-published`/`-draft` for full-width) are reserved and read by the UI; they must exist with identical keys and value shapes because apps and clients read them.
- Space properties: same model on spaces.
- App properties: per-app key/value (Forge/Connect-style app storage), the v2 `App Properties` ops `[V]` scopes `write:app-data:confluence`.
- User properties (v1): per-user key/value.
- Property search: CQL `content.property[key].path` `[V]`.

## Likes and watches

- Like: toggle per content per user; count and list of users; requires view. Like on comments also `[V]`.
- Watch: user watches a page (and optionally descendants), blog post, or whole space; `watching` state endpoints; watch implicitly added when you create/edit/comment/are mentioned per user preference `[V]`. Watchers receive change notifications ([11](11-events-jobs.md)).

## Content states and classification

- Content state: space-level set of available states, per-page assigned state (draft and published variants), history of state changes. Space setting enables/disables.
- Classification level: site-defined list; default; set per space and per content; read in lists; permission to classify gated `[V]`.

## Tasks

- Tasks are parsed from page bodies (task lists) into rows keyed by `localId`; status changes from the page toggle write the page body (new version, minor edit) `[V]`. The v2 Task API reads and updates status; update of a task writes back to the body.
- "My tasks" aggregates by assignee/creator, due date, page, space.
