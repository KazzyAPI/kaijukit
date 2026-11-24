# F03 Page view, history, restrictions, attachments, labels

Last reviewed: 2026-11-24

## Page view layout

- Optional breadcrumbs above title (ancestor chain, collapsed with "..." when long, each links), title (H1, not part of body), **byline**: author avatar+name -> profile, "last updated" relative time with exact time tooltip, reading time `[V]`, collaborators/contributors facepile `[V]`, like count and button, watcher count/eye `[V]`, analytics link (permission/setting gated) `[V]`. Optional header image/cover `[V]`. Body (viewer) with **fixed-width** (max ~760px `[V]`) or **full-width** per page property `content-appearance-published`.
- Actions row (right of title): **Edit**, **Share**, **Comment**? (jump to comments), **Watch**/Following toggle, **Star**, "..." menu.
- "..." menu items: Copy link, Page history, Page information `[V]`, Page analytics `[V]`, Export to PDF, Export to Word, Print `[V]`, Move, Copy, Archive, Delete, Restrictions, Watch/Stop watching, Page properties/Labels, View source (storage) `[V]`, Create template from page `[V]`. Each gated by `operations`.
- Below body: footer with labels (add/remove inline), reactions/likes, comments section (footer comments, F05), "attachments" shortcut. Right-hand panel: table of contents outline (sticky) `[V]` if enabled, inline-comment markers gutter.

> **Status (web app, 2026-11-25):** a header bar sits under the top bar (collapse chevron, icon, breadcrumb-style title, "Edited <date>", collaborator avatars, Edit, Share with copy-link, overflow menu). The canvas is a ~740px column with a large light title, a decorative icon tile in the cover slot (no cover upload yet), and a byline: avatar, "By <name>" (names via `POST /users-bulk`, short id until the server has it), reading time from the word count at 200 words per minute, plus version number and draft lozenge. "See views" and "Add a reaction" are disabled stubs. Share opens a dialog with the link and copy; the lock icon is always the open state because page restrictions are not built. Classic pages keep the separate edit and publish flow; the live-document model from capture 001 is not built.

## Rendered content

- Server-rendered `view` HTML hydrated: macros (TOC, expand toggles, tabs, code copy button, status lozenges), headings with anchor links (hover link icon -> copy URL with `#fragment`), tables (sticky header `[V]`, horizontal scroll), images (click -> media viewer), task checkboxes (interactive for edit permission; toggling PATCHes task status), mentions (hover profile card), smart links (inline/card/embed with hover preview), emoji, dates, panels, layouts (responsive column stack under 768px).
- Media viewer: full-screen overlay, prev/next, zoom, download, file info, PDF paging, Esc closes, focus trap.

## Page history

- `/history`: left list of versions (number, author, time, message, minor-edit marker, "current" tag) with load-more; right: selected version rendered. **Compare**: pick two versions -> rich diff of rendered HTML with additions highlighted (green) and deletions (red strike), unchanged context collapsed `[V]`, changed macros marked; prev/next change navigation. Actions: **Restore this version** (confirm + optional message -> new version), **Delete** version (admin), "Copy link". Permission: view history needs view; restore needs edit.
- Diff algorithm: block-level then inline word diff over normalized ADF/HTML; deterministic.

## Page information

- Metadata table (id, space, created by/when, last modified by/when, version count, word count `[V]`, labels, restrictions summary), "What links here" `[V]`, page properties, content state.

## Restrictions & sharing

- **Restrictions dialog**: three modes (Anyone can view/edit, Anyone can view + only some edit, Only specific people can view/edit) `[V]` labels ours; user/group picker with type-ahead, role per subject (Can view / Can edit), remove, "Apply" ; shows inherited restriction notice with link to the ancestor; lock icon in header/tree; disabled with explanation without `restrict_content`. Writes via restrictions endpoints.
- **Share dialog**: add people/emails with message -> sends notification (non-users get invite flow `[V]`); copy link; public link toggle when enabled by admin.

## Attachments

- Attachments view (`/pages/viewpageattachments.action` equivalent `[V]`): table (preview thumbnail, name, size, type, uploader, date, version, comment), upload (drag-drop + file picker, progress, multi, cancel), actions: download, delete, view versions, rename `[V]`, "Attach new version", preview. Max size error toast; blocked type error.
- Inline in editor: drag/paste uploads (see F04).

## Labels

- Inline label bar at page bottom: chips with remove x, "+ Add label" input with autocomplete (popular/suggested), validation (no spaces, lowercase), add on Enter/comma. Label click -> label results page (content by label with space filter) `[V]`.

## Likes/reactions, star, watch

- Like toggles with optimistic count + tooltip of users; star adds to Starred menu; watch menu options: Watch this page, Watch page + children, Watch space, Stop watching. Emoji reactions on page body `[V]` if present.

## Export and print

- PDF/Word export via async task; the browser downloads on completion; toasts. Print stylesheet hides chrome, expands macros (expand/tabs) `[V]`.

## States

Loading (skeleton for title, byline, body), error (retry), not found/restricted (generic page with request-access `[V]`), trashed banner with restore, archived banner, draft banner with "resume editing" `[V]`, old-version banner when viewing historical.
