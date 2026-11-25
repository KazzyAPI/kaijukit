# F03 Page view, history, restrictions, attachments, labels

Last reviewed: 2026-11-24

## Page view layout

- Optional breadcrumbs above title (ancestor chain, collapsed with "..." when long, each links), title (H1, not part of body), **byline**: author avatar+name -> profile, "last updated" relative time with exact time tooltip, reading time `[V]`, collaborators/contributors facepile `[V]`, like count and button, watcher count/eye `[V]`, analytics link (permission/setting gated) `[V]`. Optional header image/cover `[V]`. Body (viewer) with **fixed-width** (max ~760px `[V]`) or **full-width** per page property `content-appearance-published`.
- Actions row (right of title): **Edit**, **Share**, **Comment**? (jump to comments), **Watch**/Following toggle, **Star**, "..." menu.
- "..." menu items: Copy link, Page history, Page information `[V]`, Page analytics `[V]`, Export to PDF, Export to Word, Print `[V]`, Move, Copy, Archive, Delete, Restrictions, Watch/Stop watching, Page properties/Labels, View source (storage) `[V]`, Create template from page `[V]`. Each gated by `operations`.
- Below body: footer with labels (add/remove inline), reactions/likes, comments section (footer comments, F05), "attachments" shortcut. Right-hand panel: table of contents outline (sticky) `[V]` if enabled, inline-comment markers gutter.

> **Status (web app, 2026-11-25):** a header bar sits under the top bar (collapse chevron, icon, breadcrumb-style title, "Edited <date>", collaborator avatars, Edit, Share with copy-link, overflow menu). The canvas is a ~740px column with a large light title, a decorative icon tile in the cover slot (no cover upload yet), and a byline: avatar, "By <name>" (names via `POST /users-bulk`), reading time from the word count at 200 words per minute, plus version number and draft lozenge. "See views" and "Add a reaction" are disabled stubs. Share opens a dialog with the link and copy; the lock icon shows the page's restriction state from `GET /pages/{id}/restriction-state` (open or locked, with the explanation in the dialog); the restrictions dialog itself, which changes who can view or edit, is not built. Pages the user cannot read return 404 and show the not-found page. Comment authors use the top-level `authorId` (kaijukit extension), falling back to `version.authorId`; label chips link to the label's page list. Classic pages keep the separate edit and publish flow; the live-document model from capture 001 is not built.

> **Status, page features (web app, 2026-11-25):** built against the real API: like button and count with user tooltip (optimistic; `PUT/DELETE /rest/api/content/{id}/like`); label chips with add (Enter or comma, no spaces, lowercase) and remove (chips are not links: no content-by-label endpoint); attachments list, upload (picker and drag-drop), download (fetched as a blob with auth) and delete, which trashes; page history at `pages/history/{id}` with the version list, old-version view, and Restore as a `PUT` that publishes the old title and body as version current+1 (restore drops annotations missing from the old body, so their comments turn dangling; the dialog warns). Not built: Compare/diff (stub), delete version, minor-edit marker, load-more, attachment versions, rename, upload progress and cancel, label results page.

> **Status, restrictions (web app, 2026-11-25):** the Share dialog edits page restrictions through the v1 operations (`GET /content/{id}/restriction/byOperation?expand=...`, `PUT/DELETE .../byOperation/{op}/user?accountId=` and `/byGroupId/{id}`, `DELETE /content/{id}/restriction`): three levels (no restrictions, editing restricted, viewing and editing restricted), people and groups for view and edit with remove buttons and a picker (`/search/user`, `/group`). Choosing a level adds you first so the server's lockout rule is not tripped. A page the caller cannot read is a not-found page (the server answers 404); a forbidden action shows "You don't have permission to do that" with the server's reason, and a 403 page load shows a permission page. Not built: inherited-restriction display and share-by-email.

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
