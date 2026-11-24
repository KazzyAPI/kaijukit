# Milestones

Last reviewed: 2026-11-24

Each milestone ends with: all CI jobs green, conformance cases for its operations passing in `schema` mode, UI screenshots reviewed by a human or agent for bugs, inventory/checklist updated.

| # | Scope | Exit criteria |
|---|---|---|
| M0 | Workspace, CI, Docker compose, Helm skeleton, `/healthz`, migrations, auth (basic+token, bootstrap admin), conformance runner (schema mode) | `docker compose up` serves `/wiki/api/v2/spaces` (empty list); CI green; one conformance case passing |
| M1 | v2 spaces + pages (create/get/list/update/delete/trash), children/ancestors/descendants, versions; storage<->ADF for core nodes; web app: shell, spaces directory, space sidebar tree, page view, basic editor with draft/publish; Playwright e2e | create space, create page, edit, publish, view in browser; e2e green; v2 Space/Page/Version/Children ops pass schema conformance |
| M2 | Remaining ADF nodes and marks, macros core set, attachments, comments (footer+inline), labels, properties, likes | ops in those v2 tags pass schema; editor node tests |
| M3 | Permissions and restrictions, CQL + search, v1 content/search/space endpoints, notifications, jobs | restriction and CQL conformance cases |
| M4 | Remaining v2/v1 inventory, templates, export/import, admin, audit, users/groups, OIDC/SAML | inventories 100% `partial` or better |
| M5 | Realtime collaboration (Yjs), Connect-compatible layer (optional) | multi-client tests |
| ongoing | Capture reference cases, resolve `[V]` markers, move rows `partial` -> `done` | per row |
