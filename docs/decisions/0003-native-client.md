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
