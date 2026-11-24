# Confluence parity

Last reviewed: 2026-11-24

Full behavior specs: [../spec/confluence/README.md](../spec/confluence/README.md) (13 backend + 8 frontend docs). Generated API structure: [confluence-api/](confluence-api/).

## Hard requirements (non-negotiable)

These are release gates, not goals.

1. **Drop-in API compatibility.** An existing Confluence Cloud client, script, CI job or marketplace-style integration must work against kaijukit by changing only the base URL and credentials. Same paths (`/wiki/api/v2/...`, `/wiki/rest/api/...`), methods, query parameters, request/response JSON field names and types, status codes, error bodies, pagination (`_links.next`, `Link` header, `cursor`/`start`/`limit`), expansion (`expand`, `body-format`), and ordering defaults.
2. **Every operation in the pinned public OpenAPI specs is implemented.** Inventories: [v2](confluence-rest-v2.md) (252 ops), [v1](confluence-rest-v1.md) (130 ops). `scripts/confluence_inventory.py check` fails CI if the pinned spec and inventory diverge or a row is `done` without a conformance case. Operations we deliberately skip are marked `not started` with a reason and still count against parity.
3. **Product parity, not just API parity** (UI spec: [confluence-ui-parity.md](confluence-ui-parity.md); document format: [confluence-adf.md](confluence-adf.md)). Everything a user can do in the Confluence UI (see areas below) must be possible in kaijukit with equivalent behavior. Data created via the API must render in the UI identically and vice versa.
4. **Content-format parity.** Storage format (XHTML-based), the editor's ADF/Atlas document format, and `view`/`export_view`/`styled_view`/`anonymous_export_view` representations must round-trip and convert as the reference does, including macro and `ac:`/`ri:` elements.
5. **Evidence-based.** A row is `done` only when a recorded conformance case in `conformance/` replays the same request against the reference instance and kaijukit and the normalized responses match. No self-reported parity.
6. **Compatibility, not copying.** Clean-room rules in [../clean-room.md](../clean-room.md) apply. Product names appear only in docs; the shipped UI uses kaijukit names ([../naming-policy.md](../naming-policy.md)). Wire-level names (paths, JSON fields) must match.

## Integration surface (must match)

| Area | Requirement | Source |
|---|---|---|
| Base paths | `/wiki/api/v2`, `/wiki/rest/api`, plus the site-level path structure | public OpenAPI `servers` |
| Auth | Basic (email + API token), OAuth 2.0 (3LO) with granular scopes, personal access tokens for self-hosted-style deployments, plus OIDC/SAML for UI SSO. Scope names must match the spec's `securitySchemes`. | OpenAPI `components.securitySchemes` |
| Pagination | v2: cursor-based, `limit`, `Link` header and `_links.next`. v1: `start`/`limit`/`size`, `_links.next`. | OpenAPI + public docs |
| Errors | Same status codes and error JSON shapes per operation | OpenAPI `responses`; verify with conformance |
| Rate limiting | Same headers/429 semantics (`Retry-After`) so clients back off correctly | **verify against reference** |
| Webhooks / events | Event names and payloads for content/space/user events; delivery, retry and signature behavior | **TBD: capture from reference; do not guess** |
| App model | Decide whether Connect/Forge-style app descriptors, dynamic modules (v1 "Dynamic modules") and app properties are supported; required for "integrated the same way" | **open decision** |
| Search | CQL grammar and fields, `/search` and `/search/user` result shape, excerpts, highlighting | [../parity/query-parity.md](query-parity.md) (add CQL section) |

## Product areas

Entry template is in [checklist.md](checklist.md). Each area gets entries as specs are written; none exist yet.

| # | Area | Notes | Entries |
|---|---|---|---|
| 1 | Spaces | global/personal/knowledge-base types, keys, homepage, categories, themes, settings, archive, space roles | 0 |
| 2 | Pages, blog posts, folders | hierarchy (ancestors/children/descendants), move, copy, reorder, draft vs current vs trashed vs historical status, titles unique per space | 0 |
| 3 | Versions & history | version numbers, messages, minor edits, restore, diffs | 0 |
| 4 | Content body & formats | storage, ADF, view, export_view, wiki; macro model (see Macros) | 0 |
| 5 | Editor | rich-text editor, autosave drafts, **realtime collaborative editing**, mentions, emoji, tables, layouts, inline comments anchoring | 0 |
| 6 | Macros | built-in set (TOC, children, include, excerpt, status, code, info panels, jira-issue links, etc.), user macros, macro body API | 0 |
| 7 | Attachments & media | upload, versioning, thumbnails, previews, size limits, virus-scan hook | 0 |
| 8 | Comments | footer and inline comments, replies, resolve/reopen, anchoring on content change | 0 |
| 9 | Labels, properties, likes, watches | content/space properties (JSON), labels, popular labels, likes, watchers | 0 |
| 10 | Permissions & restrictions | space permissions, page view/edit restrictions, inheritance, groups, anonymous access, admin key | 0 |
| 11 | Search | CQL, site search, user search, indexing latency expectations | 0 |
| 12 | Templates & blueprints | space/global templates, blueprints, variables | 0 |
| 13 | Content states, classification, data policies, redactions | v2 Content State, Classification Level, Data Policies, Redactions | 0 |
| 14 | Whiteboards, databases, smart links, tasks | v2 Whiteboard, Database, Smart Link, Task | 0 |
| 15 | Users, groups, user properties | account IDs, groups, anonymous/unknown users, privacy | 0 |
| 16 | Audit & analytics | audit log records/retention, page view analytics | 0 |
| 17 | Long-running tasks, export/import | space export (PDF/Word/HTML/XML), import, async task polling | 0 |
| 18 | Notifications & email | watches, mentions, digests, email-to-page | 0 |
| 19 | Admin & settings | site settings, look-and-feel, themes, system info | 0 |

## Order of work

1. Content model and formats (areas 2–4), because every endpoint depends on them.
2. Conformance harness against the reference instance (record/replay + normalizer).
3. Spaces, pages, versions, attachments (read endpoints first, then writes).
4. Permissions/restrictions, search/CQL.
5. Remaining v2 and v1 operations, driven by the inventories.
6. Editor and realtime collaboration last; riskiest area.

## Risks

- Parity is only as honest as the reference instance. Nothing here has been checked against one yet.
- Published OpenAPI specs are known to lag or differ from live behavior; conformance captures win over the spec when they disagree, and the discrepancy is recorded here.
- Some Cloud behavior (rate limits, search ranking, async export, email) has no public spec; those rows need captures, and some may be unreachable. Unreachable rows stay `not started`; they are never marked `done` by assumption.
- Reproducing a hosted product's exact behavior may conflict with licensing or terms of service of the reference instance; get that reviewed before capturing at scale.
