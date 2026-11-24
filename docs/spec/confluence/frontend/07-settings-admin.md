# F07 Site settings and administration UI

Last reviewed: 2026-11-24 · backend: [04](../backend/04-permissions.md), [10](../backend/10-users-auth.md), [12](../backend/12-admin-export.md)

Layout: admin area under `/wiki/admin/...` with a left nav of groups and a content pane. Everything here is permission-gated (site admin / system admin) and writes through the settings/permissions/users endpoints; forms validate inline, show unsaved-changes guards, and success flags.

| Group | Screens |
|---|---|
| Users & groups | Users list (search, filter by status/last active/group, deactivate, reactivate, delete), user detail (groups, last active, tokens), Groups list/detail (members add/remove), invite users (email, role), bulk import CSV `[V]`, directory/SSO settings (OIDC, SAML IdP metadata, attribute mapping, JIT provisioning, SCIM token), external collaborators `[V]` |
| Permissions | Global permissions matrix (Can use, Create space, Personal space, Admin, System admin, Browse users, Anonymous), per group/user/anonymous; "Admin key" enable/disable with duration and banner while active |
| Spaces | All spaces table (name, key, type, status, last activity, size `[V]`), actions: archive, delete, change permissions, export; default space permissions; space categories |
| Content | Site-wide trash/orphaned? `[V]`, attachment limits and types, max page size, default editor mode, collaborative editing on/off, macros enabled/disabled (HTML macro), allowed embed domains, whitelists |
| Look and feel | site title, logo/favicon upload, colors/themes, announcement banner (text, color), custom CSS/JS disabled by default `[V]` |
| Security | session timeout, password policy, 2FA enforcement, anonymous access, API token policy (allow, max lifetime), OAuth apps list and authorized clients, IP allowlist `[V]`, CORS allowlist, rate-limit configuration |
| Data governance | Classification levels (create/edit/default/status), data policies (rules, exceptions), redactions/audit export |
| Audit | Audit log: filters (date range, category, user, text), table (time, user, summary, affected object, IP), detail pane with changed values, export CSV, retention setting |
| Email & notifications | SMTP config + test, from address/prefix, templates (subject/body overrides per language), digest schedule, notification defaults |
| Apps | Installed apps list, install by descriptor URL, enable/disable, config pages, app permissions/scopes; only if app model is decided in scope |
| System | System info (version, build, uptime, DB), health checks, job queue monitor (counts per type, failures, retry), search index (rebuild, status), export/import jobs, backups `[V]`, license/telemetry n/a, logs download |
| Analytics | Site-level usage (active users, views, top spaces) `[V]` |

Standard UI parts: data tables (sort, filter, pagination, row actions, bulk select), confirmation dialogs for destructive actions with typed confirmation for delete-space/user, forms with field-level errors, empty states, and permission-denied pages.

Settings read/write must produce the same API responses as the v1 Settings/Themes/Audit inventory ([parity inventory](../../../parity/confluence-rest-v1.md)).
