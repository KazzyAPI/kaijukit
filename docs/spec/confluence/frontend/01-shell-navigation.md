# F01 Shell and navigation

Last reviewed: 2026-11-24 · `[V]` = verify by capture. Parity = function, layout, interaction; not branding ([../../../parity/confluence-ui-parity.md](../../../parity/confluence-ui-parity.md)).

## Routes (must resolve identically)

| Route | Screen |
|---|---|
| `/wiki/` , `/wiki/home` | Home (for you / recent / popular / starred) |
| `/wiki/spaces` | Spaces directory |
| `/wiki/spaces/{KEY}/overview` | Space home (homepage content) |
| `/wiki/spaces/{KEY}/pages/{id}/{slug}` | Page view |
| `/wiki/spaces/{KEY}/pages/edit-v2/{id}` | Editor (existing page) |
| `/wiki/spaces/{KEY}/pages/create` `[V]` | New page in editor (query: `parentPageId`, `templateId`) |
| `/wiki/spaces/{KEY}/blog/{yyyy}/{mm}/{dd}/{id}/{slug}` | Blog post |
| `/wiki/spaces/{KEY}/whiteboard/{id}`, `/database/{id}`, `/folder/{id}` `[V]` | Other content types |
| `/wiki/spaces/{KEY}/pages/{id}/{slug}?...` + `/history`, `/info`, `/analytics` `[V]` | Page tools |
| `/wiki/spaces/{KEY}/settings/...`, `/wiki/admin/...` | Settings, admin |
| `/wiki/search?text=&cql=&spaces=...` | Search page |
| `/wiki/x/{tiny}` , `/wiki/pages/viewpage.action?pageId=` , `/wiki/display/{KEY}/{Title}` | Redirects |
| `/wiki/people/{accountId}` , `/wiki/spaces/~{accountId}` | Profile, personal space |
| `/wiki/login`, `/wiki/logout`, `/wiki/error/{code}` | Auth, errors |

Client-side routing (SPA) with history API; deep links work on full load; unauthorized -> login with `next=`; hidden content -> the same 404 page as nonexistent.

> **Superseded in part by observed capture [001](../../../../conformance/ui/captures/001-page-live-edit.md):** the reference places global destinations (For you, Recent, Starred, Spaces, Apps, Teams, More) and the space block in a left navigation panel. The top bar holds only sidebar toggle, app switcher, logo, a wide search field, Create, notifications, help and the avatar. Where this file says menus live in the top bar, follow the capture.

## Global top bar (fixed, all screens)

Left to right: (1) site switcher/app menu (ours), (2) product logo+name -> home, (3) menus **Recent**, **Spaces**, **People**, **Starred**, **Templates**, **Apps** `[V]` each opens a dropdown panel (recent list with type icon + title + space + time; "Filter by title"; "View all recent" link), (4) **Create** button (primary) -> create dialog (see F06 templates), (5) search box expands on focus into quick-search overlay, (6) right cluster: notifications bell with unread badge (panel, tabs Direct/Watching `[V]`, mark all read, settings link), help menu, settings (admin) gear, profile avatar menu (profile, personal space, tasks, drafts, preferences, appearance/theme, log out).
States per control: hover, focus ring, active, disabled (no create permission -> Create still shown with no-space-available state `[V]`), loading skeleton for dropdown lists, empty states, error states.
Keyboard: `/` or `Ctrl/Cmd+K`? focus search `[V]`; `c` create `[V]`; `g` then `h` home etc. `[V]` (single-key shortcuts toggleable in preferences); `?` opens shortcut help.

## Home

- Feed tabs: **Recent** (default), **Popular**, **Following/for you** `[V]`; items = content cards (title, space, excerpt `[V]`, author avatar, action text "updated/commented/created", relative time). Infinite scroll with cursor paging. Left rail: **Recent spaces**, **Starred**, **Drafts** `[V]`; right: onboarding cards. Empty state with call to create.
- "My work" `[V]`: tasks (assigned/created, incomplete/complete, due), drafts list (resume/delete).

## Spaces directory

- Tabs: all, favorites `[V]`, archived (if permitted), categories filter, search-by-name, sort, create-space button (permission-gated). Card/list toggle. Space card: icon, name, key, description snippet, star toggle.

## Notifications panel

- List grouped by recency; item: actor avatar, action sentence (ours), target link, time, unread dot, quick actions (reply to comment inline `[V]`). Click navigates and marks read. Pagination on scroll. "Notification settings" page: per-event toggles, digest, watch defaults.

## Profile menu & preferences

- Pages: profile (about, recent activity feed, spaces), settings (email, password, 2FA, API tokens page with create/revoke, language, time zone, editor options incl. "disable single-key shortcuts", "default page width full/fixed", theme light/dark/auto `[V]`, notification prefs, watches list, favorites, drafts, saved items).
- API tokens screen: create (name, expiry), reveal once, copy, revoke, last used.

## Errors and edge states

- 404/403-as-404, 500, offline banner, session expired modal (re-login without losing editor content), maintenance banner, license/quota banners `[V]`, rate-limited toast.

## Flags/toasts

- Bottom-left flag stack: success/info/warn/error, auto-dismiss rules, action link ("Undo", "View"), max stack, a11y live region.

## Routing/data contract

All lists use v2 endpoints where available (pages, spaces, comments), v1 for search, recent, notifications, user prefs. Each component lists its `operationId`s in its entry.
