# UI review log (M1)

Last reviewed: 2026-11-25

Method: `web/e2e/tests/review.spec.ts` drives every screen and state in four configurations (light/dark x 1280px/390px), saves a screenshot per step to `web/e2e/screenshots/` (gitignored), runs axe (WCAG 2.1 A/AA) and checks for horizontal page overflow. Screenshots were read by the author and the bugs below were fixed. Run: `pnpm e2e` (mock, includes the review spec) or `E2E_BASE_URL=http://localhost:8080 pnpm --filter @kaijukit/e2e e2e -- flow` (real stack; the review spec needs the mock's `__reset` endpoint for empty states).

Screens covered (shell rebuild of 2026-11-25 adds the notifications menu, help and shortcuts dialog, app switcher, quick search with results and no-match, Recent and Starred flyouts, tree title search with hit and no-match, tree row menu, share dialog, collapsed page header, editor text-style and insert menus, disabled-button tooltip): login, login error, home (empty, populated), spaces (empty, populated, filter no-match, archived empty), create space dialog (valid, invalid key), space overview, page with tree, page view (rich content), page actions menu, delete dialog, page loading skeleton, page error (500), page not found, sidebar collapsed/overlay, account menu, create menu, editor (new, slash menu, table, link dialog error, saved, rich content), publish dialog, duplicate-title error, discard dialog.

## Bugs found and fixed

| # | Where | Bug | Fix |
|---|---|---|---|
| 1 | Editor | Pressing Enter in the title before the lazy editor chunk loaded did not move focus; typing went into the title | Editor and its route load as one chunk; focus is synchronous (`view.dom.focus()`) |
| 2 | Editor | "State update on a component that hasn't mounted yet" warning from a forced re-render on every transaction | Removed; Tiptap already re-renders on transactions |
| 3 | Tree | Tree item accessible name included the row action ("Parent Create child page under Parent"), so names collided | `aria-labelledby` points at the label only |
| 4 | Tree | `aria-posinset`/`setsize` counted across parents | Computed per sibling group |
| 5 | Icons | "More" (...) icon rendered as two-pixel dots, nearly invisible in light and dark | Drawn as filled-looking circles |
| 6 | Breadcrumbs | On 390px the "/" separator wrapped to the start of the next line | Separator moved to `::after` of the previous item |
| 7 | Editor toolbar | Table buttons wrapped alone onto a second row at 840px | Tighter button padding |
| 8 | Editor | Contextual table tools labelled "+ Col" had accessible names that did not contain their visible text (axe label-content-name-mismatch); same for H1/H2/H3 | Visible text is contained in the accessible name ("Add column", "H1 heading") |
| 9 | Top bar | Logo link had no name at 390px (text hidden) | `aria-label="kaijukit home"` |
| 10 | Space shell | "Show sidebar" button name did not contain its visible text | Visible text is the name |
| 11 | Page view | Code blocks that scroll were not keyboard focusable (axe scrollable-region-focusable) | `tabIndex=0` on `pre`; slash list likewise |
| 12 | Publish dialog | Duplicate-title error was attached to the comment textarea, turning it red | Separate alert box above the actions |
| 13 | Page view 390px | Title squeezed beside the Edit button into a narrow column | Actions stack above the title under 768px |
| 14 | Space shell 390px | Sidebar overlay stayed open after choosing a page, covering it; the closed state was persisted and leaked into desktop | Closes on navigation under 768px; only desktop state is persisted |
| 15 | Editor 390px | Breadcrumb in the editor header wrapped into three lines | Hidden under 520px (title is directly below) |
| 16 | Editor autosave | Real backend appends a version on every PUT; client sent a stale number and Publish returned 409 | Next number is always last returned + 1; one retry after refetch on 409; a visible toast on persistent 409 |
| 17 | Editor autosave | Real backend rejects saving a draft over a published page (400) so the chip showed "Not saved" | Published pages keep edits local with an "Unpublished changes" chip, a leave confirmation, and `beforeunload`; only never-published drafts autosave |

## Shell rebuild (capture 001), 2026-11-25

| # | Where | Bug | Fix |
|---|---|---|---|
| 18 | Menus | An absolutely positioned menu inside the scrolling left panel was clipped and scrolled with it | Menus render in a portal on `body` with fixed positioning measured from the trigger; closed on outside scroll and resize |
| 19 | Menus | Row action menus rendered inside tree items gave the tree item the menu's text as content (axe label-content-name-mismatch), and keys in the menu moved tree focus | Portal (above); the menu stops key events from reaching ancestors |
| 20 | Menus | A menu holding only text labels failed `aria-required-children`, and with no item to focus Esc did nothing | Empty states are disabled menu items; the list itself takes focus when it has no focusable item |
| 21 | Toolbar | Disabled (not built) subtle buttons kept a bordered grey box and looked like pressed toggles | Subtle `aria-disabled` buttons are transparent |
| 22 | Top bar 390px | First tooltip ("Show sidebar") centered on a button at the viewport edge and was cut off | The first top bar tooltip aligns to the left edge |
| 23 | Menus | Long single-item text (notification note) wrapped with no vertical padding | Items have 6px vertical padding |
| 24 | Page header | On a space home page the breadcrumb read "Space / Space" | The space crumb is omitted when the page is the home page |
| 25 | Tests | `getByLabel('Title')` matched the "Search by title" field and tree row buttons | Specs use exact label matching |

Design decisions worth knowing: quick search and tree search list the newest 250 pages and match substrings on the client because `GET /pages?title=` is exact-match; the notifications badge is a single local welcome item that clears when opened (no fake counts); the Share lock icon is always the open state because restrictions are not built; toolbar items without a Tiptap extension are `aria-disabled` buttons with a "not built yet" tooltip, not hidden.

## Editor features and known issues (M2), 2026-11-25

Driven with Playwright (`web/e2e/tests/editor-features.spec.ts`): every feature is inserted through the toolbar or its keyboard trigger, the saved ADF is read back from the API and compared literally, then the page is screenshotted in view and edit mode, light and dark, at 1280 and 390 wide, with axe. Run against both the mock API and the real stack (`COMPOSE_PROJECT_NAME=kk-web`, port 8181).

| # | Found by looking | Fix |
|---|------------------|-----|
| 1 | Pressing Enter on an empty last action item or decision added more empty items instead of leaving the list. | Enter falls back to lifting the item out of the list. |
| 2 | Inserting an image (or any block) while a block such as the table of contents was selected replaced it. | Inserts go after a selected block node. |
| 3 | After editing an Expand title, clicking its body left the caret at the top of the page, so typing landed in the first paragraph. ProseMirror skips resyncing the DOM caret when the click does not change its own selection. | Re-focus the view after a click that left the title input. |
| 4 | New lines after aligned or indented text inherited the alignment and indent, so `[] ` produced centred action items. | Kept (it matches Word-style behaviour); the spec resets alignment before typing the list. |
| 5 | Red text on the yellow highlight failed axe colour contrast (3.9:1) in view mode. | Red swatch changed to `#bf2600` (5.5:1). |
| 6 | Dark-blue text colour is low contrast on the dark theme. | Not fixed: author-chosen colours are content, and the palette keeps ADF-compatible values. |

Title search: `GET /pages?title=` is an exact (case-insensitive) match on the title, not a prefix or substring search. Quick search and the tree filter now say "Exact page title" and, when nothing matches, "No page is titled ...". Prefix or full-text search needs a backend endpoint.

## Page features (comments, labels, likes, history, attachments, tasks, search), 2026-11-25

Driven with Playwright (`web/e2e/tests/page-features.spec.ts` for behaviour, `page-features-review.spec.ts` for screenshots, axe and horizontal-overflow checks, light and dark at 1280 and 390). Both need the real stack (`E2E_BASE_URL`).

| # | Found by looking or running | Fix |
|---|-----------------------------|-----|
| 1 | The focused inline comment was highlighted as two disjoint stripes (head and body only). | The whole comment gets the highlight. |
| 2 | Side panel and history list backgrounds stopped at their content height, leaving a hole in the column. | Fixed height with scroll. |
| 3 | At 390px the attachment row pushed Download onto its own line. | Under 520px the button is icon-only. |
| 4 | The panel scrolled back to the focused comment on every refetch, fighting resolve and the Open/Resolved toggle. | It follows a focused comment once. |
| 5 | Typing a comma in the label box committed the previous text. | Commit takes the value being typed. |
| 6 | On mobile the bottom sheet covers the footer comments. | By design: close it first (kept). |
| 7 | The floating Comment button can cover the heading above a selection at 390px. | Kept: it follows the selection and never hides it. |

Limits found while building: no comment-author field (the last editor is shown), footer comments cannot be resolved, restore drops annotations that are not in the old body, and the Vite dev proxy needed `/wiki/download`.

## Search, labels, permissions (M3 backend), 2026-11-25

Method: `page-features-review.spec.ts` (real stack, four configurations, axe plus overflow check) now also covers the lock state with the share dialog, the label results view and quick search on page text. Screenshots read: `pf-{light,dark}-{wide,narrow}-{lock-state,label-view,quick-search-text,search}.png`.

| # | Where | Bug | Fix |
|---|---|---|---|
| 1 | Search results page | The page icon shrank to a few pixels in rows with a long excerpt (flex item squeezed). | Icon is `flex: none`. |
| 2 | e2e `flow.spec.ts` shell test | Strict mode violation: the link "Search for ..." and the result link both matched the title. | Exact name match. |

Changes: quick search and `/wiki/search` use `GET /wiki/rest/api/search` with CQL (`type = page AND (title ~ "x" OR text ~ "x")`: partial title, stemmed text); the current user comes from `GET /user/current` (no more `users-bulk` scan); comment authors use the top-level `authorId` and fall back to `version.authorId`; label chips open `/wiki/labels/:id` (`GET /labels/{id}/pages`); the mention picker uses `GET /search/user` with `user.fullname ~`; the share button shows a lock (and "Share, restricted" for assistive tech) from `GET /pages/{id}/restriction-state`. Pages the user cannot read already come back as 404 and render the not-found page. Mock server gained stand-ins for search, search/user, user/current and restriction-state.

Not verified end to end: the 404 for an unreadable page (the stack has one user, so no second account to test with).

## Restrictions, space settings, groups, watches, move (M4), 2026-11-25

Method: `web/e2e/tests/access.spec.ts` on the real stack (`COMPOSE_PROJECT_NAME=kk-web4`, port 8686, Vite proxy on 5174). Four behaviour tests and four review configurations (light and dark, 1280 and 390 wide) with axe and a horizontal-overflow check; screenshots `ac-{light,dark}-{wide,narrow}-{restrictions,settings-details,settings-permissions,groups,group-delete,move-dialog}.png` and `ac-light-wide-drag-hint.png` were read by eye. A second, non-admin account (Bob) is inserted into the stack's Postgres by the spec because no endpoint creates users; with it the 404-vs-403 behaviour was verified end to end: a restricted page opens as the "We can't find that page" screen for Bob, becomes readable once he is added to the viewers, and his attempt to change the restrictions answers 403 and shows "You don't have permission to do that" with the server's reason.

| # | Where | Bug found by looking or running | Fix |
|---|---|---|---|
| 1 | Permissions grid | The row header cell used `display: flex`, which stopped it behaving as a table cell: rows with a wrapped group name got a different height and lost their bottom border, and the sticky column misaligned with the checkboxes. | Flex layout moved to a div inside the cell. |
| 2 | Move dialog | Search results from several spaces looked identical (same title, space name run on after it). | The row reads "Title in Space name". |
| 3 | e2e | `check()` on controlled radios and checkboxes fails with "did not change its state" because the state changes only after the request returns. | Specs click and then assert the state. |
| 4 | e2e | The drag assertion passed on the toast left over from the dialog move before the drag had finished. | The spec polls the API for the resulting order. |

Notes: the Share dialog still opens with the page link selected (first field is auto-focused, existing behaviour). The grid scrolls sideways at 390 wide with a sticky first column and is keyboard focusable. Space Details is read-only because no endpoint updates a space. Drag and drop is pointer only; the Move dialog is the keyboard route. Page restriction changes add the editor first because the server rejects a change that locks the caller out.

Not built: blog posts and folders in the left nav (the backend has no routes for them on main), a notification read API (the bell stays a local stub), inherited-restriction display, space roles.

## Blog posts and folders in the nav (M4), 2026-11-25

Method: `web/e2e/tests/kinds.spec.ts` on the real stack (four configurations, axe, overflow check); screenshots `kn-{light,dark}-{wide,narrow}-{folder-tree,blog-dialog,blog-view}.png` read by eye. Blogs are listed from `GET /spaces/{id}/blogposts`, created with `POST /blogposts` (title and plain text, no rich editor yet) and opened at `/wiki/spaces/:key/blog/:id`. Folders appear in the tree where a page lists them as children (type `folder`); a folder's children come from `/folders/{id}/direct-children`, selecting a folder opens it instead of navigating, and "New folder" creates one under the space home page.

| # | Where | Bug | Fix |
|---|---|---|---|
| 1 | Tree | The children probe used `/pages/{id}/direct-children`, which answers 404 for a folder, so folder contents never loaded. | A 404 falls through to the folders endpoint. |
| 2 | e2e | Folder seeded under a collapsed page was not visible, and the nav link is hidden after navigation at 390 wide (sidebar overlay closes). | Spec parents the folder to the home page and checks the nav link on wide only. |

Limits: root-level folders are not listed by any endpoint (so none are created at the root); folders cannot be deleted or moved from the UI; blog posts cannot be edited, commented or labelled yet.

## Known, not fixed

- Tree has no "load more" past 250 children and no virtualization.
- Chevrons appear only for pages with children (the tree probes children of every visible row), which costs one request per visible row.
- Search has no filters, sorting, highlighting or CQL mode yet; excerpts come from the server as plain text (text nodes are joined with spaces, so punctuation can show a gap).
- The table of contents is a static placeholder while editing; it lists headings when the page is viewed.
- Inline comments are built on published bodies (the server anchors them), so `Editor.annotate` / `removeAnnotation` are unused by the comment UI.
- Layout sections have no per-column width controls (2 and 3 equal columns only).
