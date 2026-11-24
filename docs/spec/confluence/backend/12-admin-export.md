# 12 Admin, audit, export/import, templates, analytics

Last reviewed: 2026-11-24

Structure: [v1 Audit](../../../parity/confluence-api/v1/audit.md), [v1 Template](../../../parity/confluence-api/v1/template.md), [v1 Settings](../../../parity/confluence-api/v1/settings.md), [v1 Themes](../../../parity/confluence-api/v1/themes.md), [v1 Analytics](../../../parity/confluence-api/v1/analytics.md), [v1 Space settings](../../../parity/confluence-api/v1/space-settings.md), [v2 Data Policies](../../../parity/confluence-api/v2/data-policies.md), [v2 Classification Level](../../../parity/confluence-api/v2/classification-level.md).

## Spaces

- Create (key rules: unique, 1-255 chars of letters/digits `[V]`, key immutable after create unless alias), types global/personal, archive/unarchive, delete (async, trashes content then purges), description, icon, categories/labels, homepage change, settings (e.g. routing, editor settings), theme/look-and-feel (colors, sidebar config, header/footer custom HTML limited), space roles/permissions (see 04). Space export triggers long task producing a downloadable archive.
- Space `operations` listing per caller.

## Templates and blueprints

- Templates: site-level and space-level; body with variables (`<at:var at:name="x"/>` in storage `[V]`), placeholder text instructions, labels, create-from-template sets title/body, per-template permissions to create `[V]`.
- Blueprints (built-in): meeting notes, product requirements, decision log, how-to, retrospective, file list, 3-column?, troubleshooting, shared links, etc. `[V]` capture the full list; each defines wizard fields, body, index page, label.
- Blueprint index pages and "create from template" macro use the same registry.

## Export / import

- Page export: PDF, Word (`doc`), HTML, with options (page + descendants `[V]`). Space export: HTML, XML (full backup incl. attachments), PDF. Export honors permissions (only what exporter can view) except admin XML backups.
- Import: XML space import with ID remapping, user mapping, conflict handling; site restore for admin.
- Our XML format must match the reference's schema closely enough to import reference exports `[V]` (stretch requirement, tracked separately; still **required** for migration parity).
- Async via long-running task + notification with download link; links expire.

## Audit

- Append-only records: who, when, IP, category (permissions, user management, spaces, global config, content restore/purge, apps, ...), summary, affected objects, changed values. Query with date range, text, category, author; retention configurable; export CSV; API per the v1 Audit inventory (retention get/set, search, export).

## Analytics and stats

- Page views, unique viewers per content (and per day) used by the page analytics UI and by `popular` lists; privacy rules (viewers visible to who `[V]`). Data rolled up by the analytics job; raw view events retained N days.

## Site settings (v1 Settings/system)

- System info (version, build, license-like values replaced with ours), look-and-feel colors, default space settings, date/time formats, language, email settings (SMTP), banner/announcement `[V]`, attachment size, max page size, rate limit config, whitelist, anonymous access, user directory config, security (session, password policy, 2FA), mail templates.

## Dynamic modules / apps (open decision)

- v1 Dynamic modules API registers/removes app modules for an app key. If app model is in scope, a module registry with the same endpoint semantics; else these ops are `not started` by decision (still count against parity).
