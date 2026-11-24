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

## Known, not fixed

- Page byline shows `User <short id>` until the server implements `POST /users-bulk`; the UI already uses it when present.
- Leaf pages show an expand chevron until their children have been fetched (the tree endpoint reports no child count).
- The help menu and app switcher are hidden under 768px; their items are not reachable on phones yet.
- Tree has no "load more" past 250 children and no virtualization or drag and drop.
- Breadcrumbs do not collapse long chains; they truncate and wrap.
- Toolbar tooltips can cover the first editor line.
