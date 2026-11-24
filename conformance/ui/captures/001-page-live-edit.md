# Capture 001: page open in the reference editor (user-provided screenshot, 2026-11-25)

Source: a screenshot supplied by the maintainer from a licensed instance. The image is not stored in the repo (third-party branding, sample content and illustration); only the structural observations below are, in our own words. Build identifier and date: unknown, record when re-captured.

## Layout (1x, ~1840px wide window)

1. **Top bar** (white, ~56px): left to right: sidebar collapse toggle, app switcher (grid icon), product logo + name, **wide search field** (about 40% of the width, centered-left, magnifier icon, placeholder "Search"), primary **Create** button (blue, plus icon), then at the right a notifications bell with a numeric red badge, help, and the user avatar. No Recent/Spaces/Starred dropdowns in the top bar.
2. **Far-left strip**: thin dark navy vertical band (~30px) at the window edge.
3. **Left navigation panel** (~300px, light gray-white, resizable/collapsible), two groups:
   - Global: For you, Recent (chevron), Starred (chevron), Spaces (chevron), Apps, Teams, More (…). Icon + label rows, chevron opens a flyout list.
   - Current space block: space avatar + space name + "…" menu; **Shortcuts**; **Content** (row with + and … actions); a **"Search by title"** field; the **page tree** (chevron or bullet per row, a content-type icon per row, truncated titles with ellipsis, active row highlighted with a light-blue fill and a blue left bar); a **Create** row under the tree; **Blogs** (+), "No blogs in this space" empty text; **Calendars**; **Space apps** (… menu) with app entries listed below.
4. **Content header bar** (under the top bar, ~50px, bottom border): collapse chevron, content-type icon, page title (breadcrumb-style, bold, truncated), at the right "Edited <date>", a collaborator avatar with a badge, a **Share** button group (lock icon + "Share" and a link-copy icon), and a "…" overflow menu.
5. **Formatting toolbar** (one row, always visible while the doc is editable): text style dropdown ("Tt"), bold, italic (+ dropdown), text color, highlight color, alignment dropdown, list dropdown, action item (checkbox), link, image, mention (@), emoji, layout columns, table (+ dropdown), insert (+ dropdown), and a pin toggle at the far right.
6. **Document canvas**: centered column ~ 740px wide, generous top margin; very large light-weight **title** (~32px) in the canvas, an optional **cover/emoji illustration** floating at the right of the title, then a **byline row**: author avatar + "By <name>", reading time with a book icon, "See views" with a chart icon, "Add a reaction" with a smiley-plus icon, then secondary page-detail items; body text ~16px with 1.6 line height, paragraphs separated by ~16px.
7. The scrollbar is a thin overlay at the canvas edge.

## Behavior inferred from the capture `[V]`
- The page is shown in the **editor with the toolbar visible and no Publish button**, plus "Edited <date>": this is the live-document model, where view and edit are the same surface and changes autosave and publish continuously. The tree icon for this page differs from classic pages (live-document icon). Classic pages still use a separate edit/publish flow.
- Sharing is a first-class header action with a lock state indicator.
- Space navigation and page tree live in the **left panel**, not the top bar. Global destinations are in the same panel.

## Deltas against our current web app (to fix)
| Area | Ours now | Reference |
|---|---|---|
| Top bar | Logo, "Spaces" menu, Create, avatar | toggle, switcher, logo, large search, Create, bell with badge, help, avatar |
| Left nav | none global; space sidebar only | global destinations + space block (Shortcuts, Content, tree, Blogs, Calendars, Space apps) |
| Tree | titles only | content-type icons, bullets/chevrons, search-by-title, active row bar, + and … row actions, Create row |
| Page header | title in body | header bar with icon, title, edited date, collaborators, Share, link, overflow |
| Editing | separate edit route + Publish dialog | live-doc in-place editing for live docs, toolbar in page; classic pages keep draft/publish |
| Toolbar | H1/H2/H3 buttons, icons | text style dropdown, color/highlight, alignment, list dropdown, action item, mention, emoji, layouts, table, insert menu, pin |
| Byline | "User <id>" | avatar, By <name>, reading time, See views, Add a reaction |
| Canvas | narrow, small title | ~740px column, large light title, cover/illustration slot |

## Do not copy
Logo, product name, icons, illustration, the sample content and names. Our wording, our icons, our tokens (see naming policy and ADR 0002).
