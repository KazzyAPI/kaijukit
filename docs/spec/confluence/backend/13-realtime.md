# 13 Realtime collaboration

Last reviewed: 2026-11-24

Highest-risk area; implement last, specified now so earlier choices do not block it.

- **Model:** a collaborative session per draft document. CRDT = Yjs-compatible (`yrs`) over WebSocket (`/wiki/collab/{contentId}`), awareness for presence and selections. Document projected to/from ADF through a ProseMirror <-> Yjs binding using a schema generated from the ADF schema.
- **Modes:** per-site/space setting "collaborative editing" on (default) or off (legacy: single editor with edit-lock/conflict warnings `[V]`). Both must work; legacy mode is needed for parity with reference settings.
- **Lifecycle:** first editor opens draft -> session created from latest published body (or existing draft). Edits stream and persist (debounced snapshots + update log). Publish snapshots the CRDT doc to ADF, creates a version, and keeps the session open for continued editing as a new draft. Last editor leaves -> session closes after idle timeout, draft remains.
- **Presence:** avatars of current editors (max shown `[V]`), colored remote cursors with name labels, selection highlights, "N people editing" state, viewer vs editor distinction.
- **Offline/reconnect:** local buffered updates merged on reconnect; banner states for offline/reconnecting/synced; conflict-free merge, no "your changes will be lost" except on explicit revert.
- **Permissions:** session join requires edit access to the content; restriction changes mid-session evict the user; read-only viewers can follow live (view-only) `[V]`.
- **API interplay:** `PUT /pages/{id}` while a session is active: applies as a new published version based on the live doc state and resets the session doc, or rejects with 409 per reference `[V]` (capture to decide). Version history labels versions produced by multi-author sessions with all contributing authors `[V]`.
- **Comments in sessions:** inline comment markers are part of the CRDT as annotation marks so anchors follow concurrent edits.
- **Scale:** sessions pinned to a node via consistent hashing, with Postgres as durable store and LISTEN/NOTIFY or Redis pub/sub to fan out between nodes; target 50 concurrent editors per doc (reference limit `[V]`).
- **Security:** session tokens short-lived, bound to user + content; rate-limit updates; max update size; validate doc against ADF schema before persisting to published body.
- **Tests:** deterministic multi-client simulations (property tests) on the core CRDT/ADF projection; Playwright multi-browser tests for presence and merge.
