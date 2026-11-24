# 03 Body formats

Last reviewed: 2026-11-24

Representations requested with `body-format` (v2) or `expand=body.<repr>` (v1).

| Representation | Direction | Notes |
|---|---|---|
| `storage` | read + write | XHTML-based, namespaces `ac:` (app content) and `ri:` (resource identifier). Canonical for API clients and legacy apps. |
| `atlas_doc_format` | read + write | ADF JSON (string-encoded in the `value`). Canonical for the new editor. Schema: [../../../parity/confluence-adf.md](../../../parity/confluence-adf.md). |
| `view` | read | rendered HTML with macros executed, as shown to the viewer |
| `export_view` | read | like view, with absolute URLs, for export/email |
| `styled_view` | read | view plus inline styles `[V]` |
| `anonymous_export_view` | read | export_view rendered as an anonymous user `[V]` |
| `editor`, `editor2` | read (v1) | editor-ready XHTML `[V]` |
| `wiki` | write (v1, legacy) | legacy wiki markup `[V]` converted on write |
| `raw` | custom content only | opaque app payload |

Response body object: `{ representation, value }`; v2 returns the requested representation under `body.<repr>`, and `body` is `{}` unless `body-format` is given.

## Conversion rules

1. Source of truth is ADF. Writing `storage` converts storage -> ADF; reading `storage` converts ADF -> storage. Round-trip storage -> ADF -> storage must be semantically identical (not byte-identical: attribute order and whitespace normalized) for every construct in the corpus in `conformance/body/`.
2. Unknown storage elements and macros are preserved as ADF `extension`/`bodiedExtension`/`inlineExtension` nodes and re-emitted verbatim. Nothing is dropped silently; unsupported content is preserved and flagged.
3. Macro in storage: `<ac:structured-macro ac:name="x" ac:schema-version="1" ac:macro-id="uuid"><ac:parameter ac:name="p">v</ac:parameter><ac:rich-text-body>...</ac:rich-text-body> | <ac:plain-text-body><![CDATA[...]]></ac:plain-text-body></ac:structured-macro>`. `ac:macro-id` is preserved across edits.
4. Links: `<ac:link><ri:page ri:content-title="T" ri:space-key="K"/><ac:plain-text-link-body><![CDATA[text]]></ac:plain-text-link-body></ac:link>`; other `ri:` targets: `ri:attachment ri:filename`, `ri:user ri:account-id`, `ri:space ri:space-key`, `ri:blog-post ri:content-title ri:posting-day`, `ri:url ri:value`, `ri:shortcut`. Anchors via `ac:anchor`.
5. Images: `<ac:image ac:width ac:height ac:align ac:border ac:alt ac:title ...><ri:attachment ri:filename="x.png"/></ac:image>` or `ri:url`.
6. Tasks: `<ac:task-list><ac:task><ac:task-id/><ac:task-uuid/><ac:task-status>incomplete|complete</ac:task-status><ac:task-body>..</ac:task-body></ac:task></ac:task-list>` map to ADF taskList/taskItem and to the Task resources.
7. Layouts: `<ac:layout><ac:layout-section ac:type="two_equal|two_left_sidebar|two_right_sidebar|three_equal|three_with_sidebars|single"><ac:layout-cell>..` map to layoutSection/layoutColumn with widths.
8. Mentions: `<ac:link><ri:user ri:account-id="..."/></ac:link>`; emoji `<ac:emoticon ac:name="smile" ac:emoji-shortname ac:emoji-id ac:emoji-fallback>`; dates `<time datetime="YYYY-MM-DD"/>`; status lozenge is the `status` macro; placeholder `<ac:placeholder>`.
9. Inline comments: `<ac:inline-comment-marker ac:ref="uuid">text</ac:inline-comment-marker>` maps to ADF `annotation` mark (`annotationType: inlineComment`, `id`). The marker uuid links to the inline comment's `markerRef`.
10. Tables: HTML `table/tbody/tr/th/td` with `data-layout`, `data-highlight-colour` and colwidth `colgroup`; ADF table attrs `isNumberColumnEnabled`, `layout`, `displayMode`, cell `colspan/rowspan/colwidth/background`.
11. Sanitization: on write, strip scripts, event handlers, `javascript:` URLs; HTML macro is disabled by default `[V]`. Storage input is validated: malformed XML is a 400 with a parser message.
12. Size: 5 MB per request (413).

## Rendered `view`

- Executes macros server-side (TOC, children, include, content by label, page properties report, etc., see [09](09-macros.md)), resolves links (title -> id, broken links styled as unresolved), applies restrictions (macro output must not leak restricted content), embeds attachments, and returns HTML using stable class names that the frontend CSS targets. Class names we expose are ours, not copied.
- Output is cached per (content, version, viewer-class); invalidated on any dependency change (included page, label, attachment).

## Links tracking

- On every write, extract outgoing links and referenced attachments/users (mentions) into index tables for: "What links here" `[V]`, mention notifications, rename rewriting, orphan detection, and the `mention`/`macro` CQL fields.

## Test corpus

`conformance/body/` holds paired inputs and the reference's outputs for each construct above, in every direction (storage<->ADF<->view). A construct is `done` only when all directions match under normalization.
