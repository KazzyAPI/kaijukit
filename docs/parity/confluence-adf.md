# Confluence document format (ADF) — node and mark inventory

Last reviewed: 2026-11-24

Source: <http://go.atlassian.com/adf-json-schema> (sha256 `5128562b75278c8a83e7e3619a570205bc80d59696985ec31a7a7883cff66fbe`). Names below are the schema's definition names; variants such as `*_with_marks`/`*_full` are the same node in another context. Hard requirement: every node and mark must parse, validate, round-trip losslessly through the API, render in the editor and viewer, and convert to/from storage format as the reference does.

Status: `not started` | `partial` | `done` (needs a conformance/UI case).

## Nodes

| Node | Parse/validate | Render | Edit | Storage-format conversion | Case |
|---|---|---|---|---|---|
| `blockCard` | not started | not started | not started | not started | - |
| `blockTaskItem` | not started | not started | not started | not started | - |
| `blockquote` | not started | not started | not started | not started | - |
| `bodiedExtension` | not started | not started | not started | not started | - |
| `bodiedExtension_with_marks` | not started | not started | not started | not started | - |
| `bodiedSyncBlock` | not started | not started | not started | not started | - |
| `bulletList` | not started | not started | not started | not started | - |
| `caption` | not started | not started | not started | not started | - |
| `codeBlock` | not started | not started | not started | not started | - |
| `codeBlock_root_only` | not started | not started | not started | not started | - |
| `code_inline` | not started | not started | not started | not started | - |
| `date` | not started | not started | not started | not started | - |
| `decisionItem` | not started | not started | not started | not started | - |
| `decisionList` | not started | not started | not started | not started | - |
| `doc` | not started | not started | not started | not started | - |
| `embedCard` | not started | not started | not started | not started | - |
| `emoji` | not started | not started | not started | not started | - |
| `expand` | not started | not started | not started | not started | - |
| `expand_root_only` | not started | not started | not started | not started | - |
| `extension` | not started | not started | not started | not started | - |
| `extension_with_marks` | not started | not started | not started | not started | - |
| `formatted_text_inline` | not started | not started | not started | not started | - |
| `hardBreak` | not started | not started | not started | not started | - |
| `heading` | not started | not started | not started | not started | - |
| `heading_with_alignment` | not started | not started | not started | not started | - |
| `heading_with_indentation` | not started | not started | not started | not started | - |
| `heading_with_no_marks` | not started | not started | not started | not started | - |
| `inline` | not started | not started | not started | not started | - |
| `inlineCard` | not started | not started | not started | not started | - |
| `inlineExtension` | not started | not started | not started | not started | - |
| `inlineExtension_with_marks` | not started | not started | not started | not started | - |
| `layoutColumn` | not started | not started | not started | not started | - |
| `layoutSection` | not started | not started | not started | not started | - |
| `layoutSection_full` | not started | not started | not started | not started | - |
| `listItem` | not started | not started | not started | not started | - |
| `media` | not started | not started | not started | not started | - |
| `mediaGroup` | not started | not started | not started | not started | - |
| `mediaInline` | not started | not started | not started | not started | - |
| `mediaSingle` | not started | not started | not started | not started | - |
| `mediaSingle_caption` | not started | not started | not started | not started | - |
| `mediaSingle_full` | not started | not started | not started | not started | - |
| `mention` | not started | not started | not started | not started | - |
| `nestedExpand` | not started | not started | not started | not started | - |
| `nestedExpand_with_no_marks` | not started | not started | not started | not started | - |
| `orderedList` | not started | not started | not started | not started | - |
| `panel` | not started | not started | not started | not started | - |
| `paragraph` | not started | not started | not started | not started | - |
| `paragraph_with_alignment` | not started | not started | not started | not started | - |
| `paragraph_with_font_size` | not started | not started | not started | not started | - |
| `paragraph_with_indentation` | not started | not started | not started | not started | - |
| `paragraph_with_no_marks` | not started | not started | not started | not started | - |
| `placeholder` | not started | not started | not started | not started | - |
| `rule` | not started | not started | not started | not started | - |
| `status` | not started | not started | not started | not started | - |
| `syncBlock` | not started | not started | not started | not started | - |
| `table` | not started | not started | not started | not started | - |
| `table_cell` | not started | not started | not started | not started | - |
| `table_header` | not started | not started | not started | not started | - |
| `table_row` | not started | not started | not started | not started | - |
| `taskItem` | not started | not started | not started | not started | - |
| `taskList` | not started | not started | not started | not started | - |
| `text` | not started | not started | not started | not started | - |
| `text_with_no_marks` | not started | not started | not started | not started | - |

## Marks

| Mark | Parse/validate | Render | Edit | Storage-format conversion | Case |
|---|---|---|---|---|---|
| `alignment` | not started | not started | not started | not started | - |
| `annotation` | not started | not started | not started | not started | - |
| `backgroundColor` | not started | not started | not started | not started | - |
| `border` | not started | not started | not started | not started | - |
| `breakout` | not started | not started | not started | not started | - |
| `code` | not started | not started | not started | not started | - |
| `dataConsumer` | not started | not started | not started | not started | - |
| `em` | not started | not started | not started | not started | - |
| `fontSize` | not started | not started | not started | not started | - |
| `fragment` | not started | not started | not started | not started | - |
| `indentation` | not started | not started | not started | not started | - |
| `link` | not started | not started | not started | not started | - |
| `strike` | not started | not started | not started | not started | - |
| `strong` | not started | not started | not started | not started | - |
| `subsup` | not started | not started | not started | not started | - |
| `textColor` | not started | not started | not started | not started | - |
| `underline` | not started | not started | not started | not started | - |
