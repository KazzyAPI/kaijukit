# Confluence UI parity

Last reviewed: 2026-11-24

## Hard requirements

1. **Functional and interaction parity.** Every screen, control, shortcut, state and flow a Confluence user has must exist in kaijukit and behave the same: same information architecture, same navigation, same keyboard shortcuts, same editor behavior, same empty/error/loading/permission-denied states.
2. **Layout parity.** A user of the reference should find every control where they expect it. Same regions (global nav, space sidebar, page header, content area, right-hand panels, modals), same responsive breakpoints and mobile behavior.
3. **Not a visual clone.** Parity ends at branding. We do not copy logos, marks, icons, illustrations, fonts that are licensed, UI copy text, or help-doc prose ([../clean-room.md](../clean-room.md), [../naming-policy.md](../naming-policy.md)). We use our own icon set, wording and theme tokens that achieve the same function and similar density. Banned names are enforced in the UI by `scripts/check-naming.sh`.
4. **Accessibility is part of parity.** Keyboard operability for every flow and screen-reader semantics at least as good as the reference. WCAG 2.1 AA is the floor.
5. **Evidence.** A UI row is `done` only with a recorded UI case: a scripted browser test (Playwright) that runs the flow, plus a reference capture (screenshot or DOM/ARIA snapshot or screen recording) stored in `conformance/ui/<case>/` and compared by a human reviewer on first recording, then by automated regression. No capture, no `done`.
6. **Same data, same result.** Content authored in the UI must produce the same API/ADF/storage output as the reference ([confluence-adf.md](confluence-adf.md), [confluence-parity.md](confluence-parity.md)), and vice versa.

## How to write an entry

Each surface below gets entries in this form (added to its own `confluence-ui/<surface>.md` as it is specced; the table here is the index):

```
### <control or flow>
- Behavior: plain language, our words (what the user does and what happens).
- Source: public support-doc URL or conformance/ui/<case> capture.
- States: default, hover/focus, loading, empty, error, disabled, no-permission.
- Keyboard & a11y: shortcuts, focus order, roles/labels.
- API calls it makes: operationIds from the v1/v2 inventories.
- Status: not started | partial | done
- Tests: conformance/ui/<case>, component tests
```

## Surfaces

Public user docs for behavior: <https://support.atlassian.com/confluence-cloud/> (read for behavior only, never copy prose). Everything not covered by public docs must come from observed behavior on a licensed reference instance.

| # | Surface | What it covers | Entries |
|---|---|---|---|
| 1 | Global shell | top nav, create button, search box and quick search, recent/starred/for-you menus, notifications, profile and help menus, app switcher, site-level routing and URLs | 0 |
| 2 | Home & feeds | home dashboard, recently worked on, popular, activity feed, tasks (my tasks), drafts | 0 |
| 3 | Spaces directory & space shell | space list, create space (blank/templates), space sidebar, page tree (drag/drop, expand/collapse, resize), space overview/home, shortcuts, quick actions | 0 |
| 4 | Page view | title, byline (author, edited date, reading time, likes, watchers), breadcrumbs, table of contents, content viewer, reactions/likes, share, watch, star, page actions menu, full-width toggle, print/export | 0 |
| 5 | Page editor | toolbar, `/` insert menu, `@` mentions, `:` emoji, `[[`/link picker, markdown shortcuts, tables (resize, merge, header row/col, sort, number columns), layouts/columns, panels, expand, code block, status, date, decisions, action items/tasks, media (drag, paste, resize, captions), smart links/embeds, macros (insert, configure, edit, placeholder), full-width, comment-on-selection, undo/redo, find & replace, keyboard shortcuts | 0 |
| 6 | Draft, publish & autosave | draft lifecycle, publish dialog (version comment, notify watchers, minor edit), unpublished changes banner, discard, conflict handling | 0 |
| 7 | Realtime collaboration | presence avatars, remote cursors/selections, concurrent edits, offline/reconnect, version labelling of collaborative sessions | 0 |
| 8 | Comments | footer comments, inline comments (highlight, anchor, orphaned), replies, resolve/reopen/delete, mentions in comments, comment panel and filters | 0 |
| 9 | Page history & versions | version list, view old version, compare/diff, restore, revert, version comments, page information, page analytics | 0 |
| 10 | Page tree operations | create child, move (dialog and drag), copy (with children), reorder, delete/trash, archive, restore from trash | 0 |
| 11 | Restrictions & sharing | page restriction dialog (view/edit, users/groups), inherited restrictions indicator, share-by-link/email | 0 |
| 12 | Attachments & files | attachments list, upload, preview (images, PDF, Office), versions of a file, file macros, media viewer | 0 |
| 13 | Labels & properties | add/remove labels, label pages, popular labels, content by label | 0 |
| 14 | Search | quick search overlay, full search page, filters (space, contributor, type, label, date), result cards, advanced (CQL) search | 0 |
| 15 | Templates & blueprints | template picker, create/edit template (variables), built-in blueprints (meeting notes, how-to, retrospective, etc.), index pages | 0 |
| 16 | Space settings | details, permissions, roles, look-and-feel, themes/sidebar config, content tools (trash, archive, export/import), integrations, automation, apps | 0 |
| 17 | Site admin | users/groups, global permissions, anonymous access, spaces admin, site settings, audit log, data policies, classification, security/SSO config, email, apps/marketplace-equivalents, rate-limit/usage | 0 |
| 18 | Notifications & email | in-app notification list, watch settings, email digests and templates, mention emails, per-user preferences | 0 |
| 19 | Whiteboards, databases, smart links, folders | the v2 Whiteboard, Database, Smart Link, Folder objects in the UI | 0 |
| 20 | Profile & personal | profile page, personal space, preferences (language, timezone, editor mode), API tokens, saved filters | 0 |
| 21 | Responsive & mobile web | breakpoints, collapsed sidebar, touch editing, mobile-specific menus | 0 |
| 22 | Cross-cutting | error pages (404/403/500), session expiry, loading skeletons, toasts/flags, i18n and RTL, dark/light themes, keyboard shortcut help, undo of destructive actions | 0 |

## Editor specifics

The editor is the highest-risk surface. Its document model is ADF ([confluence-adf.md](confluence-adf.md)); the web editor is built on ProseMirror/Tiptap with a schema generated from the ADF JSON schema so that nodes/marks cannot drift. Realtime collaboration (surface 7) uses Yjs/yrs. Per-node editing behavior (what Enter, Backspace, Tab, paste and drag do inside each node) is specified per node in the editor entries and tested with scripted keystroke cases.

## Order of work

1. Global shell, space shell, page view (read-only), driven by v2 read endpoints.
2. Editor core: paragraph, heading, lists, marks, links, tables. Then remaining ADF nodes.
3. Draft/publish, versions, comments.
4. Restrictions, attachments, labels, search.
5. Templates, space settings, site admin.
6. Realtime collaboration last.

## Risks

- UI parity has no public spec. Everything depends on observed behavior from a reference instance, and the reference UI changes continuously. Each capture records the date and visible build identifier; re-review periodically.
- Pixel-level likeness is explicitly out of scope (requirement 3). If "same look" is wanted beyond layout and density, get a legal opinion first; trade dress claims are a risk for open-source clones.
- The user-facing support docs are copyrighted. Use them to learn behavior, not to copy text.
