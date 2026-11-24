# Confluence full spec

Last reviewed: 2026-11-24

Complete behavior specs for the Confluence parity target ([../../parity/confluence-parity.md](../../parity/confluence-parity.md)). Read order: backend 01 to 12, then frontend 01 to 08.

- **Structure** (paths, params, bodies, responses, scopes, schemas) is generated from the pinned public OpenAPI: [../../parity/confluence-api/](../../parity/confluence-api/). Do not duplicate it here.
- **Behavior** (what the structure cannot say) is written here in our own words.
- `[V]` marks a statement written from public knowledge that has **not** been verified against the reference. A `[V]` must be resolved by a conformance capture before the covering row can be `done`. Unmarked statements are taken from the pinned spec.
- Status of every feature lives in the inventories and checklists, not here.

| Backend | Frontend |
|---|---|
| [01 data model](backend/01-data-model.md) | [01 shell & navigation](frontend/01-shell-navigation.md) |
| [02 content lifecycle](backend/02-content-lifecycle.md) | [02 spaces & page tree](frontend/02-spaces-tree.md) |
| [03 body formats](backend/03-body-formats.md) | [03 page view](frontend/03-page-view.md) |
| [04 permissions](backend/04-permissions.md) | [04 editor](frontend/04-editor.md) |
| [05 search & CQL](backend/05-search-cql.md) | [05 collaboration & comments](frontend/05-collab-comments.md) |
| [06 API conventions](backend/06-api-conventions.md) | [06 search, templates, notifications](frontend/06-search-templates-notifs.md) |
| [07 attachments & media](backend/07-attachments.md) | [07 settings & admin](frontend/07-settings-admin.md) |
| [08 comments, labels, properties](backend/08-metadata.md) | [08 design system, a11y, i18n, responsive](frontend/08-design-a11y.md) |
| [09 macros](backend/09-macros.md) | |
| [10 users, groups, auth](backend/10-users-auth.md) | |
| [11 notifications, events, webhooks, jobs](backend/11-events-jobs.md) | |
| [12 admin, audit, export/import, templates](backend/12-admin-export.md) | |
| [13 realtime collaboration](backend/13-realtime.md) | |
