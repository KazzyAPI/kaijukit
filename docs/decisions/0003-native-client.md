# ADR 0003: Native Rust desktop client

Last reviewed: 2026-11-24 · status: accepted direction, toolkit pending spike

## Decision
Besides the web app, kaijukit ships a **native desktop client written in Rust** with its own UI (not a web view, so not Tauri). It is a second client of the same API at feature parity with the web app, and shares the Rust core crates (`kk-core`, `kk-adf`, later `kk-cql`, `kk-perm`) natively instead of through WASM.

## Why
A web view in a Rust frame does not give the native feel the product owner wants (platform menus, text input and selection behavior, scrolling, accessibility, startup time, memory).

## Consequences
- Two UIs to keep at parity. The parity checklists in `docs/spec/confluence/frontend/` are the shared acceptance criteria; each row gains a `native` status next to `web`.
- The editor is the largest cost: ADF block/inline editing, tables, macros, mentions, comments, undo, IME, and later realtime (yrs on the client). Built on a text-layout crate, not a web view.
- The client talks only to the public API (`/wiki/api/v2`, `/wiki/rest/api`); no private endpoints.
- Offline cache and drafts stored in SQLite on the client.

## Toolkit selection (spike, evidence not opinion)
Candidates: Slint, Iced, egui (+ AccessKit). Prototype the hardest screen on each: render a page from ADF (headings, lists, table, code, marks) and a text-input editor with selection, IME, undo and a screen reader pass. Criteria: platform-native look and behavior, accessibility, text editing quality, scrolling and resize performance, license compatibility with Apache-2.0, binary size, build time, ecosystem health. The ADR is amended with the result.

## Layout
`native/` Cargo workspace member crates: `kk-client` (API client + models, no UI), `kk-native` (app). Spikes live in `native/spikes/` and are deleted after the decision.

## Spike result (2026-11-25)

Prototypes: `native/spikes/{egui,iced,slint}/` (own Cargo projects with an empty `[workspace]`, not root members; `native/spikes/sample.json` is the shared ADF document, `tree.txt` the page tree). Each renders the sample with `kk-adf` via a path dependency (headings, strong/em/code/link marks, nested bullet and ordered lists, code block, blockquote, 3x3 table with header row, rule), has a resizable left page tree with Up/Down navigation, a scroll pane, and an editable multiline text area. Host: macOS 14, 8 cores, Rust 1.100, versions egui/eframe 0.37 (+AccessKit), iced 0.14.0, Slint 1.19.1.

Screenshots were taken by each app itself (egui `request_screenshot`, iced `window::screenshot`, Slint `Window::take_snapshot`) on the real window at logical 1000x700 and 420x700 (Retina, so PNGs are 2x): `native/spikes/screenshots/{egui,iced,slint}-{1x,narrow}.png`. All six were viewed.

### Scores (1 = poor, 5 = strong)

| Criterion | egui + AccessKit | Iced | Slint |
|---|---|---|---|
| Native look and behavior on macOS | 2: custom-drawn widgets, bundled non-system font until we load one; menus, text-input shortcuts and scrolling are not platform ones | 2: custom-drawn; system sans resolves by name; no native menus | 3: custom-drawn but ships a macOS-like style; native menu bar API exists (unverified here) |
| Text editing (selection, IME, undo) | 4: `TextEdit` has caret, mouse/keyboard selection, copy/paste, built-in undoer; winit IME preedit/commit forwarded (egui-winit `on_ime`) | 3: `text_editor` has selection, copy/paste and IME preedit/commit; **no undo** (spike keeps whole-text snapshots, loses caret) | 4: `TextEdit` has undo/redo items, preedit handling, selection |
| Rich text rendering (inline marks, tables) | 4: `LayoutJob` per paragraph gives per-run font/color/underline/background; no bold/italic faces ship, had to load system TTC; tables via `Grid` (no borders) | 4: `rich_text` spans give per-run font, color, underline, background, links; built-in `table` widget exists; spans are **not selectable** | 3: `StyledText` (markdown subset) renders marks and has mouse selection; ADF marks must be serialized to markdown and escaped; inline code font mismatched, table columns misaligned across rows, headings not bold, no underline/color per span beyond HTML tags |
| Accessibility | 4: AccessKit wired in eframe; `TextEdit` emits `MultilineTextInput` with text selection nodes (source); not exercised with VoiceOver | 1: no AccessKit crate in the dependency tree (0 of 419 crates); no accessibility in README/ROADMAP | 4: AccessKit via winit backend (8 accesskit crates in lock), `accessible-*` properties in the language; not exercised with VoiceOver |
| Scrolling/resize | 3: smooth in the spike by inspection only; narrow window handled but sidebar kept its width | 3: `pane_grid` drag-resize works; narrow sidebar clipped labels ("Architecture", dates) | 3: retained mode, wraps correctly; sidebar fixed at 200 px at narrow width, content overlapped its scrollbar slightly |
| License vs Apache-2.0 | 5: MIT OR Apache-2.0 | 5: MIT | 2: GPL-3.0-only OR Royalty-free-2.0 OR commercial. Royalty-free is usable for a desktop app but requires showing `AboutSlint` in the About screen or a badge on a public page, and is a non-OSI license that does not make our binary Apache-only (`LICENSES/` in the crate) |
| Release binary size | 14.8 MB (11.7 MB stripped) | 11.9 MB (9.1 MB stripped) | 17.3 MB (14.1 MB stripped) |
| Clean release build | 141 s | 239 s | 122 s (wall time, `cargo clean`, `--offline`, 8 cores, sources already downloaded) |
| Ecosystem health | 5: egui 0.37.0 on 2026-10-15, 10 releases in 12 months, ~32.5k stars, eframe ~26M crate downloads; AccessKit active (macOS adapter 2026-10-25) | 3: 0.14.0 on 2025-12-07 is the only release in 12 months (0.13 was 2024-09); repo still active (push 2026-11-19), ~32k stars | 5: 1.19.1 on 2026-11-24, 10 releases in 12 months, commercial company behind it, ~24.7k stars |

Other measured numbers: resident memory five seconds after launch 46 MB (egui), 31 MB (iced), 37 MB (Slint), from `ps -o rss`. Dependency counts from `Cargo.lock`: 427 / 419 / 554 crates.

### What looked wrong in the screenshots

- egui: default font is not the system font; table has no cell borders and only partial row striping; sidebar width does not shrink with the window (at 420 px it takes about 130 px). Otherwise marks, lists, code, quote, rule and editor rendered correctly once bold/italic faces were loaded from `Helvetica.ttc`; without them `strong` is only a color change.
- Iced: first render showed bold text in a monospace face (family weight lookup with the default font resolved to a monospace bold); fixed by naming a font family on each span. Setting the same name through `default_font` made the editor text disappear (cause not found). Sidebar clipped labels at narrow width. Blockquote bar and table borders needed hand-built containers.
- Slint: list items and wrapped paragraphs were centered until `horizontal-alignment: left` was set on every `StyledText`; table columns do not line up between rows (each row is its own layout, needs a grid); inline code uses a small serif-looking face; heading size works but weight needs markup; at narrow width the fixed 200 px sidebar leaves 220 px for content.

### Not verified (state this before relying on the table)

- No keyboard, mouse-drag, IME, copy/paste or undo interaction was exercised at runtime; the editing scores come from widget source and docs. Up/Down tree navigation was implemented in all three but not driven.
- Screen reader behavior was not tested: scripted UI inspection (`osascript` System Events) hung, most likely on a missing macOS Accessibility permission. Scores reflect whether the accessibility stack is present, not that it works well.
- Scrolling and resize smoothness were not measured (no frame timing); "idle CPU" showed 0.0% on all three but `ps` reports a lifetime average.
- Slint's default macOS style and native menu support were not checked; the "native look" column is from screenshots, not side-by-side with AppKit.
- Build times are one run each on a warm network cache; the first egui build (about 101 s) overlapped with dependency downloads.

### Recommendation

Evidence does not support a clean winner. egui + AccessKit is the best fit on the criteria that the editor cares about most and that can be checked here: permissive license, built-in undo, IME events, AccessKit text-selection nodes, active releases, and per-run text styling that maps directly from ADF marks. Iced is ruled out on accessibility (no AccessKit at all) and has no undo and a slow release cadence. Slint is strong on accessibility and health but carries a license obligation and an unresolved ADR question (it is not Apache-compatible as a pure dependency; legal review needed), and its rich text goes through a markdown subset that fights ADF's marks. Recommend **egui with eframe + AccessKit**, conditional on the follow-up checks below; Slint is the fallback if native feel and accessibility outrank license cleanliness and legal accepts the Royalty-free terms.

Risks:
- egui is immediate-mode: a ProseMirror-grade editor (block-level selection across paragraphs, tables, drag handles, comments) must be built on `TextEdit`/`galley` ourselves; `TextEdit` is one widget per text block, so cross-block selection and undo are ours to write. This is the largest cost and was not prototyped.
- Not native: no system menus, text-input behavior (for example Option-arrow word movement, dictation, spell check) and look will differ from AppKit; the product owner's "native feel" goal is only partly met by any candidate.
- Accessibility: AccessKit is present but unproven here; tables and rich text exposure to VoiceOver must be tested before commit.
- Fonts: bold/italic and CJK fallback need system font loading (done for Helvetica only in the spike).
- Follow-ups before deleting the spikes: drive keyboard/IME/undo with an accessibility-permitted harness, run VoiceOver over the egui and Slint spikes, get a legal read on Slint's Royalty-free license, and prototype cross-paragraph selection on egui.
