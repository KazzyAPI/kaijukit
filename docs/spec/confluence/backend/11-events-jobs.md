# 11 Notifications, events, webhooks, jobs

Last reviewed: 2026-11-24

## Event bus

- Every write emits a domain event in the same DB transaction (outbox table) and `NOTIFY` wakes workers. Events: `page_created`, `page_updated`, `page_trashed`, `page_restored`, `page_moved`, `page_removed`, `page_viewed`?, `blog_*`, `comment_created/updated/removed/resolved`, `attachment_*`, `space_created/updated/removed/permissions_updated`, `label_added/removed`, `user_created/deactivated/reactivated`, `group_*`, `content_restrictions_updated`, `search_performed`? `[V]` exact names and payloads from reference captures.
- Delivery to: notification builder, search indexer, webhook dispatcher, audit writer, cache invalidator, analytics, realtime service.
- Ordering: per-content ordered, at-least-once, idempotent consumers (event id).

## Notifications

- Triggers: mention (page/comment), comment on watched/authored page, reply to your comment, page edited/created in a watched space/page, task assigned/due `[V]`, restriction granted, share, page published with "notify watchers", inline comment resolved, "page moved/deleted".
- Preferences: per user (notify on my own actions, on-edit/on-comment, digest frequency, email vs in-app, per-space watch). Site defaults.
- Batching: edits within a window are batched into one notification `[V]` window; minor edits suppressed.
- Channels: in-app (list with unread count, mark read/all read), email (HTML+text, per-language templates, unsubscribe link, reply-by-email for comments `[V]`), later chat integrations.
- Permission-safe: never notify a user about content they cannot view.
- Templates are ours (wording), structure matches.

## Webhooks

- Reference supports app webhooks (Connect/Forge events) rather than a simple REST webhook registry for Cloud `[V]`. Parity requirement: the same event names and payload shapes. Our deliverable: (a) a webhook registry API under `/wiki/ext/webhooks` for outbound subscriptions; (b) event payloads identical to the reference's app-event payloads; (c) HMAC signature header, retries with exponential backoff (schedule `[V]`), delivery log, dead-letter.
- **Open decision:** implement the Connect/Forge app lifecycle (install, `installed` callback, JWT auth `[V]`) for true "integrated the same way".

## Jobs (Postgres queue: `SELECT ... FOR UPDATE SKIP LOCKED`)

| Job | Purpose |
|---|---|
| index | update full-text index after writes |
| extract | attachment text extraction, thumbnails, previews |
| scan | optional antivirus |
| notify | build + send notifications and emails |
| webhook | deliver webhooks with retry |
| export | space/page export (PDF, Word, HTML, XML) |
| import | space import (XML/zip) |
| copy/move | large tree operations (long-running tasks) |
| purge | trash retention |
| digest | scheduled digest emails |
| analytics | roll up views |
| draft-gc | remove abandoned drafts |

- Each job: id, type, payload, attempts, run_after, locked_by, status; retry policy per type; visible in admin job UI.
- Long-running task API reflects job progress.
