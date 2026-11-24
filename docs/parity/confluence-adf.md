# Confluence document format (ADF) — node and mark inventory

Last reviewed: 2026-11-24

Source: <http://go.atlassian.com/adf-json-schema> (sha256 `5128562b75278c8a83e7e3619a570205bc80d59696985ec31a7a7883cff66fbe`). Names below are the schema's definition names; variants such as `*_with_marks`/`*_full` are the same node in another context. Hard requirement: every node and mark must parse, validate, round-trip losslessly through the API, render in the editor and viewer, and convert to/from storage format as the reference does.

Status: `not started` | `partial` | `done` (needs a conformance/UI case).

## Nodes

| Node | Parse/validate | Render | Edit | Storage-format conversion | Case |
|---|---|---|---|---|---|
| `blockCard` | partial | not started | not started | partial | kk-adf tests |
| `blockTaskItem` | not started | not started | not started | not started | - |
| `blockquote` | not started | not started | not started | not started | - |
| `bodiedExtension` | partial | not started | not started | partial | kk-adf tests |
| `bodiedExtension_with_marks` | not started | not started | not started | not started | - |
| `bodiedSyncBlock` | not started | not started | not started | not started | - |
| `bulletList` | not started | not started | not started | not started | - |
| `caption` | not started | not started | not started | not started | - |
| `codeBlock` | not started | not started | not started | not started | - |
| `codeBlock_root_only` | not started | not started | not started | not started | - |
| `code_inline` | not started | not started | not started | not started | - |
| `date` | partial | not started | not started | partial | kk-adf tests |
| `decisionItem` | partial | not started | not started | partial | kk-adf tests |
| `decisionList` | partial | not started | not started | partial | kk-adf tests |
| `doc` | not started | not started | not started | not started | - |
| `embedCard` | not started | not started | not started | not started | - |
| `emoji` | partial | not started | not started | partial | kk-adf tests |
| `expand` | partial | not started | not started | partial | kk-adf tests |
| `expand_root_only` | not started | not started | not started | not started | - |
| `extension` | partial | not started | not started | partial | kk-adf tests |
| `extension_with_marks` | not started | not started | not started | not started | - |
| `formatted_text_inline` | not started | not started | not started | not started | - |
| `hardBreak` | not started | not started | not started | not started | - |
| `heading` | not started | not started | not started | not started | - |
| `heading_with_alignment` | not started | not started | not started | not started | - |
| `heading_with_indentation` | not started | not started | not started | not started | - |
| `heading_with_no_marks` | not started | not started | not started | not started | - |
| `inline` | not started | not started | not started | not started | - |
| `inlineCard` | partial | not started | not started | partial | kk-adf tests |
| `inlineExtension` | partial | not started | not started | partial | kk-adf tests |
| `inlineExtension_with_marks` | not started | not started | not started | not started | - |
| `layoutColumn` | partial | not started | not started | partial | kk-adf tests |
| `layoutSection` | partial | not started | not started | partial | kk-adf tests |
| `layoutSection_full` | not started | not started | not started | not started | - |
| `listItem` | not started | not started | not started | not started | - |
| `media` | partial | not started | not started | partial | kk-adf tests |
| `mediaGroup` | partial | not started | not started | partial | kk-adf tests |
| `mediaInline` | not started | not started | not started | not started | - |
| `mediaSingle` | partial | not started | not started | partial | kk-adf tests |
| `mediaSingle_caption` | not started | not started | not started | not started | - |
| `mediaSingle_full` | not started | not started | not started | not started | - |
| `mention` | partial | not started | not started | partial | kk-adf tests |
| `nestedExpand` | partial | not started | not started | partial | kk-adf tests |
| `nestedExpand_with_no_marks` | not started | not started | not started | not started | - |
| `orderedList` | not started | not started | not started | not started | - |
| `panel` | partial | not started | not started | partial | kk-adf tests |
| `paragraph` | not started | not started | not started | not started | - |
| `paragraph_with_alignment` | not started | not started | not started | not started | - |
| `paragraph_with_font_size` | not started | not started | not started | not started | - |
| `paragraph_with_indentation` | not started | not started | not started | not started | - |
| `paragraph_with_no_marks` | not started | not started | not started | not started | - |
| `placeholder` | not started | not started | not started | not started | - |
| `rule` | not started | not started | not started | not started | - |
| `status` | partial | not started | not started | partial | kk-adf tests |
| `syncBlock` | not started | not started | not started | not started | - |
| `table` | not started | not started | not started | not started | - |
| `table_cell` | not started | not started | not started | not started | - |
| `table_header` | not started | not started | not started | not started | - |
| `table_row` | not started | not started | not started | not started | - |
| `taskItem` | partial | not started | not started | partial | kk-adf tests |
| `taskList` | partial | not started | not started | partial | kk-adf tests |
| `text` | not started | not started | not started | not started | - |
| `text_with_no_marks` | not started | not started | not started | not started | - |

## Marks

| Mark | Parse/validate | Render | Edit | Storage-format conversion | Case |
|---|---|---|---|---|---|
| `alignment` | partial | not started | not started | partial | kk-adf tests |
| `annotation` | partial | not started | not started | partial | kk-adf tests |
| `backgroundColor` | partial | not started | not started | partial | kk-adf tests |
| `border` | not started | not started | not started | not started | - |
| `breakout` | not started | not started | not started | not started | - |
| `code` | not started | not started | not started | not started | - |
| `dataConsumer` | not started | not started | not started | not started | - |
| `em` | not started | not started | not started | not started | - |
| `fontSize` | not started | not started | not started | not started | - |
| `fragment` | not started | not started | not started | not started | - |
| `indentation` | partial | not started | not started | partial | kk-adf tests |
| `link` | not started | not started | not started | not started | - |
| `strike` | partial | not started | not started | partial | kk-adf tests |
| `strong` | not started | not started | not started | not started | - |
| `subsup` | partial | not started | not started | partial | kk-adf tests |
| `textColor` | partial | not started | not started | partial | kk-adf tests |
| `underline` | partial | not started | not started | partial | kk-adf tests |
