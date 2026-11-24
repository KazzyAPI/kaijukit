# F04 Editor

Last reviewed: 2026-11-24 · highest-risk surface. Document model: ADF ([../../../parity/confluence-adf.md](../../../parity/confluence-adf.md)); web editor = our `@kaijukit/editor` on ProseMirror/Tiptap with a schema generated from the ADF JSON schema ([ADR 0002](../../../decisions/0002-own-stack.md)). The tables below are its acceptance tests. `[V]` = verify by capture.

## Screen

- Route `/wiki/spaces/{KEY}/pages/edit-v2/{id}` (and `create`). Top bar replaced by editor header: space + breadcrumb, **status of draft** ("Draft", "Saving...", "Saved", "Offline"), collaborator avatars, **Preview** `[V]`, **Close**/**Update**/**Publish** buttons, "..." menu (Page settings: restrictions, labels, properties, "Discard changes", "Page history", "Move", "Copy link", Full width toggle, "Edit page appearance" `[V]`).
- Title field: multiline auto-grow, Enter moves focus to body, placeholder text, required to publish, uniqueness validated on publish with inline error.
- Toolbar (sticky): text style dropdown (Paragraph, Heading 1-6), Bold, Italic, Underline `[V]`, "more formatting" (Strikethrough, Code, Subscript, Superscript, Clear formatting), text color, bullet list, numbered list, task list, alignment, indent/outdent, link, image/file, mention, emoji, table, **Insert** menu (+) with items, Layouts, undo/redo, find/replace, comment-on-selection button, keyboard shortcut help.
- Floating selection toolbar on text selection with: style, bold, italic, link, comment, more `[V]`; node-specific floating toolbars for tables, images, macros, panels, code blocks, expand, status, date.
- Widths: fixed (~760px) or full-width per page appearance `[V]`.

## Insert (slash `/` and toolbar `+`) items

Text: Paragraph/headings 1-6, Quote, Divider, Code snippet, Panel (info/note/success/warning/error/custom `[V]`), Expand, Decision, Action item (task), Table, Layout sections (2 columns, 3 columns, sidebars), Status, Date, Mention (`@`), Emoji (`:`), Link, Image/File (upload, from Confluence `[V]`), Media from URL, Smart link/embed, Table of contents, Excerpt, Page properties, Jira/issue link, Gallery `[V]`, "View all macros"/Macro browser (search, categories, preview, configure), Template picker `[V]`, Whiteboard/Database embeds `[V]`, Roadmap `[V]`, Chart `[V]`, Loom/video embed `[V]`. Slash menu filters on typing, arrow/Enter selects, Esc closes, grouped, recent items.

## Markdown / autoformat shortcuts

| Type | Result |
|---|---|
| `# `..`###### ` at line start | Heading 1-6 |
| `* ` `- ` `+ ` | bullet list |
| `1. ` | ordered list |
| `[] ` | action item |
| `<> ` | decision |
| `> ` | quote |
| ` ``` ` + Enter (optionally language) | code block |
| `---` (+Enter) | divider |
| `**t**` `*t*`/`_t_` `~~t~~` `` `t` `` | bold, italic, strike, inline code |
| `:name:` / `:` + text | emoji picker |
| `@` + name | mention picker |
| `//` | date picker `[V]` |
| ``[text](url)`` | link |
| URL + space/Enter | auto link (smart link per setting) |
| `\|\|` + Enter `[V]` | table |
Undo (Cmd/Ctrl+Z) immediately after an autoformat reverts it.

## Keyboard shortcuts `[V]` (Mac/Win)

Bold Cmd/Ctrl+B, Italic +I, Underline +U, Strike +Shift+S, Code +Shift+M? `[V]`, Link +K, Paragraph +Alt+0, Heading n +Alt+n, Bullet +Shift+8, Numbered +Shift+7, Quote +Shift+9, Code block +Shift+\\? `[V]`, Undo/Redo +Z / +Shift+Z (Win +Y), Select all +A, Find +F, Comment +Alt+M, Indent Tab / Outdent Shift+Tab (lists, tables), Save/publish +S (publishes/closes dialog `[V]`), Esc closes menus, `Cmd/Ctrl+/` shortcut help. Table: Tab/Shift+Tab next/prev cell, adds row at end; Alt+arrows move rows `[V]`.

## Node behaviors (per ADF node; each a testable entry)

| Node | Create | Enter | Backspace | Tab | Paste/Drag | Floating toolbar/options |
|---|---|---|---|---|---|---|
| paragraph | default | split | merge with previous / unwrap | n/a (list indent in lists) | rich paste converts HTML->ADF, plain keeps text | alignment, indent |
| heading | menu/md | new paragraph below | at start -> paragraph | - | keeps level | anchor link copy |
| bulletList/orderedList/listItem | md/menu | new item; empty item exits list | at start outdents then unwraps | Tab nests (max depth `[V]`), Shift+Tab outdents | paste list merges | ordered start number `[V]` |
| taskList/taskItem | `[] ` | new item; empty exits | outdent/delete | Tab nests `[V]` | | assign (`@`), due date `//`, checkbox toggle (ctrl-click?) |
| decisionList/decisionItem | `<> ` | new decision | | | | |
| blockquote | `> ` | newline in quote; Enter twice exits | | | | |
| codeBlock | ``` | newline (Shift+Enter exits? `[V]`) | at start unwrap | inserts tab/indents | keeps text only | language picker (searchable), copy, wrap toggle, line numbers `[V]`, delete |
| rule | `---` | | delete selects | | | |
| table/tableRow/tableHeader/tableCell | menu/md | newline in cell | | next cell | paste table, paste cells into selection | row/col insert/delete, merge/split, header row/col toggle, numbered column, cell background, sort by column, distribute columns, table width (fixed/wide/full), delete table, resize handles with snapping, fixed-layout colwidth |
| panel | menu | newline; Enter on empty exits | at start unwrap | | | type switch, color, emoji icon `[V]`, remove |
| expand/nestedExpand | menu | | | | | title edit, expand toggle persists default state, remove |
| layoutSection/layoutColumn | menu | | | | drag nodes in/out | add/remove column, column widths (2,3 col presets), delete section |
| mediaSingle/media/mediaGroup/mediaInline | upload, paste image, drag file, menu | | delete selects then deletes | | paste image, drag reorder, resize handles (keeps ratio, snaps to column), align left/center/right/wide/full-width | caption `[V]`, alt text, link, original size, replace, download, delete |
| mention | `@` | | deletes whole token | | | hover card in view |
| emoji | `:` | | deletes whole token | | | skin tone `[V]` |
| date | `//` | | | | | calendar popup, keyboard entry, format by user locale |
| status | menu | | | | | text, color (6), open to edit via click |
| inlineCard/blockCard/embedCard | paste URL | | | | | toggle inline/card/embed, open, unlink to plain URL, preview data from link metadata service |
| extension/bodiedExtension/inlineExtension (macros) | menu / macro browser | newline in body for bodied | | | | edit (opens macro config), delete, "default view/ placeholder", inline editing of rich body |
| hardBreak | Shift+Enter | | | | | |
| placeholder | template instruction | | | | | replaced on typing |

## Marks
strong, em, underline, strike, code (excludes other marks except link `[V]`), link (hover popup: edit, open, unlink; ctrl+click opens), subsup, textColor (palette ~20 colors + custom hex `[V]`), backgroundColor, alignment, indentation, breakout (wide/full-width for code/layout), annotation (inline comment highlight, click opens comment panel), border `[V]`, fontSize, fragment/dataConsumer (internal, preserved).

## Insert link dialog

Text + URL fields; search of pages/attachments by title with recent results; create link to heading/anchor; "Remove link"; Esc closes; validation of URL schemes (allow http/https/mailto/tel; block javascript:).

## Mentions
`@` opens picker: recent collaborators first, then search (user search endpoint, privacy-aware), up to ~10 results, keyboard navigable, shows avatar+name+handle; inserts mention node; **notifies on publish** (not on draft) `[V]`; mentioning someone without view access prompts to share/restrict check `[V]` dialog.

## Media upload
Drag/paste/select -> placeholder with progress -> `media` node with `fileId`; failures show inline error with retry; limit and type errors; image optimization/resizing thumbnails; paste from clipboard images; alt text prompt `[V]`.

## Draft, autosave, publish
- Autosave every ~N seconds `[V]` of idle / on blur, status chip updates; offline queue. Leaving with unsaved: draft kept ("Resume editing" in drafts list); browser beforeunload only if sync pending.
- **Publish/Update** flow: if page unchanged `Update` disabled; dialog optional "Add a version comment" and "Notify watchers" checkbox (default on/off `[V]`), Minor edit `[V]`. Validation: title required, duplicate title error. After publish: redirect to view with flag "Published".
- **Close/Discard**: "Discard changes" removes the draft after confirm; closing keeps the draft.
- **Conflict handling** (non-collab mode): warn if another user published newer version, offer merge/overwrite `[V]`. With collab mode see F05.
- **Preview** `[V]` toggles read-only rendering of the draft.
- **Page appearance** full-width toggle; **content width** persists as property.
- **Find & replace**: Cmd/Ctrl+F in editor opens bar with next/prev, case-sensitivity, replace/replace all, match count.
- **Paste handling**: from Word/Google Docs/web: sanitized to ADF; plain-text paste (Cmd+Shift+V); markdown paste converts `[V]`; URL paste -> smart link; Confluence-to-Confluence paste keeps macros and media.
- **Drag handles** on blocks for reordering `[V]`; block selection with Esc `[V]`.
- **Undo/redo** stack is per user in collab mode.
- **Limits**: body size 5 MB; doc depth; table size; message when exceeded.
- **Accessibility**: toolbar is `role=toolbar` with roving tabindex; Alt+F9 focuses toolbar `[V]`; editor is a labelled textbox; announcements for formatting changes; all dialogs focus-trapped.

## Test plan
Per node: Playwright keystroke script (type, Enter, Backspace, Tab, paste HTML fixtures) asserting the resulting ADF JSON equals the reference's for the same script (recorded under `conformance/ui/editor/<node>/`).
