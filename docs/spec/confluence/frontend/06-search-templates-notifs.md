# F06 Search, templates and blueprints, notifications UI

Last reviewed: 2026-11-24 · backend: [05 search](../backend/05-search-cql.md), [12 admin](../backend/12-admin-export.md), [11 events](../backend/11-events-jobs.md)

## Quick search (overlay)
- Focus search box -> dropdown: **recent** pages/spaces (before typing), then typed results grouped (Pages & blog posts, Spaces, People `[V]`) updated with debounce ~250ms `[V]`; each row: type icon, title with highlighted match, space name, last-updated; arrow keys navigate, Enter opens, Shift+Enter or "Search for X" opens full results; Esc closes; no-result and error states.

## Search results page
- Query box, **filters** (left or top): content type, space (multi), contributor (user picker), last modified (range presets + custom), labels, ancestors/`in page tree` `[V]`, include archived `[V]`; sort by relevance/last modified; result cards: icon, title (highlighted), breadcrumb path (space > ancestors), excerpt with highlights, author avatar, updated time, labels; attachments show file icon + parent; users show profile card. Pagination or infinite scroll, result count. **Advanced search**: toggle to CQL text mode with syntax highlighting, validation errors inline with position, autocomplete of field names/operators/values (spaces, labels, users), "Basic" toggle converts back when possible. Saved searches not in reference `[V]`.
- Empty/error states; query persisted in URL.

## Templates and blueprints UI
- **Create dialog** (from top bar Create and from space tree "+"): left categories (All, Blank page, Blog post, then blueprints `[V]` list, space templates, site templates), search, preview pane with description and sample image, space selector, "Create" button; opens editor with `?templateId=`. **Blueprint wizards**: multi-step dialog for fields (e.g. meeting notes: title, date, attendees via mention, goals), result page + index page and label automatically.
- **Template editor**: in editor mode with template banner: template name, description, variables (insert variable dialog: name, type text/multiline/list `[V]`), instruction placeholders, labels, "Save template", "Preview". Template list at space and global settings with create/edit/copy/delete, permission to create `[V]`.
- Create-from-template macro button renders in pages.

## Notifications & watches UI
- Bell panel (F01); **watch** control states (page, page+children, space); **Following** list under Profile > Settings > Watches with unfollow/bulk; email notification content structure: header, actor line, content snippet with comment text, "View page"/"Reply" CTAs, footer with "manage notifications" and "unwatch" links, plain-text alternative, mention emails stress the mention; digest emails group by space `[V]`.
- Notification settings page: toggles for "notify me when I edit", "notify on mention", "on comment replies", "on task assigned", digest frequency, language.

## Tasks page
- "Tasks" list: Assigned to me / Created by me, filter by status/due/space/page, sort, inline checkbox toggle (updates page body), link to the page and anchor, empty states.

## Drafts, recent, starred
- Drafts list (resume, discard), Recent (full page with filters Pages, Blogs, Comments, Spaces `[V]`), Starred.
