# 09 Macros

Last reviewed: 2026-11-24

Macro = named server-rendered component embedded in a body as `ac:structured-macro` (storage) / `extension` node (ADF, extensionType `com.atlassian.confluence.macro.core`, extensionKey = macro name `[V]`) with parameters and optional body (rich text or plain text).

## Contract

- Registry entry: `name`, title, description (ours), category, parameters (name, type, required, default, enum, alias), body type (`none`, `rich-text`, `plain-text`), output type (`block`, `inline`), editor behavior (inline-editable body, configurable via property panel), `view` renderer, `export` renderer, permission behavior.
- Execution: given (parameters, body, page context, viewer) return safe HTML. Renderers are pure Rust in the core crate where possible; those needing DB (children, search) go through a context trait.
- Errors: a macro that fails renders an inline error block, never fails the page. Unknown macros render a placeholder showing the macro name; content preserved.
- Macro parameters are validated; invalid values fall back to defaults like the reference `[V]`.
- Apps can register macros (dynamic modules / custom content); see app-model decision in the parity doc.

## Built-in set (target parity list; each needs a parameter table from the reference `[V]`)

| Group | Macros |
|---|---|
| Formatting | `info`, `note`, `tip`, `warning` (panels), `panel`, `expand`, `code`, `noformat`, `quote`?, `section`+`column` (legacy layout), `anchor`, `status`, `html`/`widget` (restricted), `excerpt`, `excerpt-include` |
| Navigation | `toc`, `toc-zone`, `children`, `pagetree`, `pagetreesearch`, `breadcrumbs`?, `navmap`, `include`, `viewfile`/`view-file`, `multimedia`, `gallery`, `attachments`, `recently-updated`, `recently-updated-dashboard`, `blog-posts`, `contentbylabel`, `popular-labels`, `labellist`, `listlabels`, `spaces`/`spacedetails`, `contributors`, `contributors-summary`, `profile`, `userlister`, `network`? |
| Reporting | `page-properties` (`details`), `page-properties-report` (`detailssummary`), `content-report-table`, `tasks-report`, `create-from-template`, `create-space-button`?, `livesearch`, `search`, `roadmap`, `jira`/`jira-issues` (links to the sibling issue tracker) |
| Dynamic | `date`/`time`, `emoji`, `mention`, `decision`, `status`, `loremipsum`, `cheese` |
| Charts | `chart`, `table-chart`? |

`?` = not confirmed to exist in current Cloud `[V]`. The final list is whatever the reference's macro browser lists; capture it.

## Cross-cutting behavior

- **Include/excerpt-include** pull content from other pages subject to the viewer's permissions; recursion guarded (depth limit and cycle detection with the reference's error text `[V]`).
- **Page properties** extract a key/value table from a `details` macro; the report macro queries pages by label/CQL, columns, sorting, pagination, with permission trimming.
- **TOC** builds from headings with `minLevel`, `maxLevel`, `style`, `type` (list/flat), `outline`, `include`/`exclude` regex, `separator`, `indent`, `class`; anchors derive from heading text via an exact slug algorithm `[V]` (must match so deep links keep working).
- **Status** lozenge: title, colour enum (grey, red, yellow, green, blue, purple) + subtle.
- **Code:** language list, theme, line numbers, collapse, first line, title; syntax highlighting server-side or client-side with identical class output not required, visual equivalence required.
- **Jira link macro:** renders issue keys/JQL with status via the sibling product API (when configured) else a plain link.
- **Unsupported-in-ADF macros** are carried as `bodiedExtension`.

## Macro browser data

`GET` registry for the UI (names, titles, categories, parameter forms) — served from the same registry, so the macro browser and renderer cannot drift.
