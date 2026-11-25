# F02 Space shell, page tree, space settings

Last reviewed: 2026-11-24

## Space shell layout

Three regions below the top bar: **space sidebar** (resizable 240-420px `[V]`, collapsible with a toggle and keyboard `[`), **content area**, optional **right panel** (comments, page info, outline `[V]`). Sidebar structure (top->bottom): space header (icon, name, key menu: Overview, Settings via "..." `[V]`), quick links: **Overview** (homepage), **Blog**, **Whiteboards**, **Databases** `[V]`, **Calendars** `[V]`, custom shortcuts (admin-configurable links), **Pages** tree, **Archived pages** link, bottom: **Space settings** (permission-gated) and "Create" `+`.

> **Status (web app, 2026-11-25):** the space block lives in the left nav panel: space avatar, name and menu; Shortcuts (empty, not built); Content with create and menu actions; search by title; page tree with content-type icons, chevrons or bullets, active-row bar, per-row create-child and menu (edit, copy link); a Create row; Blogs, Calendars and Space apps as empty states. Folder, whiteboard and database icons render from the `type` the API returns, but those types cannot be created yet. Not built: reorder, drag and drop, space settings, tree virtualization, archived pages link.

> **Status, moving pages (web app, 2026-11-25):** pages move with `PUT /rest/api/content/{id}/move/{before|after|append}/{target}`. The Move dialog (tree row menu and page actions menu) searches for the destination (CQL) and places the page inside it, before it or after it; this is also the keyboard route. Tree rows can be dragged with the pointer: the upper quarter of a row drops before, the lower quarter after, the middle inside (dashed outline). Errors from the server (the home page or a descendant as a target) show as a toast. Not built: reordering by keyboard inside the tree, editing, commenting and labelling blog posts, deleting or moving folders, root-level folders (no list endpoint).

> **Status, blogs and folders (web app, 2026-11-25):** the Blogs section lists blog posts and "+" creates one (title and plain text); posts open read-only at `/wiki/spaces/:key/blog/:id`. Folders show in the tree, open on select, and "New folder" creates one under the home page.

## Page tree

- Nodes: any content type in the hierarchy with icon by type; expand/collapse chevron (lazy loads children via v2 children endpoint, paged with "load more"); current page highlighted and path auto-expanded; restricted pages show lock icon `[V]`; drafts hidden.
- Interaction: click navigates (SPA); right-click or hover "..." menu: Create child page, Copy link, Move, Copy, Archive, Delete, Restrictions, Watch `[V]`; **drag-and-drop**: drop on node = make child, drop between = reorder, with a drop indicator line, auto-expand on hover, auto-scroll, cancel with Esc, optimistic update with rollback + toast on failure, keyboard alternative (cut/move via menu). Multi-select not supported `[V]`.
- Keyboard: up/down moves focus, right expands, left collapses/moves to parent, Enter opens, `F2`? rename not supported `[V]`; ARIA tree roles (`tree`, `treeitem`, `aria-expanded`, `aria-level`).
- Performance: virtualized rendering for thousands of nodes; children fetched per level; sidebar width/expanded state persisted per user per space.
- Empty states: new space with only homepage; no permission to see children.

## Page-level hierarchy actions (dialogs)

| Action | Dialog content | Result |
|---|---|---|
| Create child | opens editor with parent preset | draft created, URL create/edit-v2 |
| Move | "Move page" dialog: target search/tree picker with space selector, position (Append, Before, After) with sibling picker, "Move to" preview, warns about restrictions/links; async if large | v1 move endpoint or v2 equivalent |
| Copy | title (default "Copy of ..."), target space+parent, options: copy attachments, permissions, labels, **include descendants** with title find/replace prefix; progress bar for long tasks | copy long task |
| Delete | confirms; counts descendants; options: "Delete this page only"/"also delete children"? `[V]` (Cloud moves children up to be root/under parent `[V]`) | trash |
| Archive | confirm; archived content hidden from tree, searchable via filter | archive endpoint |
| Restore (trash/archive) | list in space settings with search, restore selected | restore |
| Reorder | alphabetical sort option for children `[V]` (the "Reorder pages" dialog, sort by title, drag) | positions |

## Space overview/homepage

- Renders the homepage page view; "Space details", "Recently updated" sidebar blocks, "Space contributors" `[V]`; customizable homepage via macros only.

## Space settings (tabs, left nav)

| Tab | Content |
|---|---|
| Space details | name, key (read-only), description, homepage selector, categories/labels, icon upload, status, **Archive space**, **Delete space** (typed confirmation) |
| Permissions / roles | matrix of subject x operation with group/user rows, add subject picker, anonymous row, toggles per cell, "Save all", conflicts warnings; roles view `[V]` |
| Content tools | Export (format picker, options, queue), Import `[V]`, Trash (list, filter by user/type, restore, purge), Archive (list, unarchive), Orphaned pages `[V]`, Templates list/create/edit, Content states config, Look and feel, Themes, Sidebar/shortcut config, Integrations (apps), Automation `[V]` |
| Look and feel | colors for header/sidebar/links, custom header/footer snippet (sanitized), theme selection, preview |
| Automation / apps | stubbed unless app model implemented |
- All saves optimistic with inline validation; leaving with unsaved changes prompts.

## Create space flow

- Dialog/page: choose type (blank, documentation `[V]`, knowledge base `[V]`, team space `[V]`), name, key (auto-suggested, editable, validated live for uniqueness), permission preset (open / private / public), template-specific options. On submit, redirect to the new space's overview. Errors inline.
