# F05 Collaboration and comments

Last reviewed: 2026-11-24 · backend: [13 realtime](../backend/13-realtime.md), [08 metadata](../backend/08-metadata.md)

## Realtime editing UI

- **Presence**: avatars in editor header (max ~5 then "+N"), each with a stable color; hover shows name; same colors for remote cursor (caret + name flag that fades after a few seconds idle) and selection overlay `[V]`. In **view mode** a "N people editing" indicator appears with link to join `[V]`.
- **Status chip**: Synced / Saving / Offline - changes will sync / Reconnecting; blocking error state offers "Copy my changes"/reload.
- **Merge behavior**: concurrent inserts/deletes merge; formatting attributes last-writer-wins per character; table structure edits tolerate concurrent row/col ops; node deletion by another user while you are inside it moves your cursor to the nearest valid position.
- **Publish while others are editing**: publishing is allowed; other editors continue seamlessly on the same draft; banner "Published by X" `[V]`.
- **Version labelling**: version list shows all contributing authors for a collaborative version.
- **Legacy (non-collab) mode**: edit-conflict banner "X is also editing" with options; implemented behind the site setting.

## Comments UI

### Footer comments
- Section at bottom of page: count heading, editor-lite input ("Write a comment...", expands to full comment editor with toolbar subset, mentions, emoji, attachments), **Save**/**Cancel**. Thread tree with replies (indent), actions: Reply, Edit (own), Delete (own/admin, confirm), Like, Copy link (anchor `#comment-{id}`), timestamp tooltip, "edited" marker `[V]`. Sorting newest/oldest `[V]`. Deleting parent confirms thread deletion.
- Keyboard: Cmd/Ctrl+Enter saves; Esc cancels with discard confirm if non-empty.

### Inline comments
- **Create**: select text (in editor draft or view) -> floating "Comment" button (and Cmd/Ctrl+Alt+M) -> side comment composer anchored to the selection (right panel on desktop, bottom sheet on mobile). Selection highlighted (yellow/gold, darker on focus `[V]`). Save writes the annotation, which on a **published page** modifies body without bumping version number? `[V]` (reference does not create a user-visible version; capture).
- **View**: highlighted text with click -> opens comment thread in side panel aligned to the highlight; overlapping highlights listed together; **comment panel** lists all inline comments in document order with filters: Open/Resolved, by author `[V]`; counts badge on the toolbar.
- **Resolve**: button on thread; highlight removed; moves to Resolved list; **Reopen** there. Orphaned (dangling) show a "Text was deleted" note and can only be viewed/deleted `[V]`.
- **Concurrency**: highlights follow edits; deleting highlighted text marks threads dangling.
- **Mentions** in comments: picker, notify, permission check.
- **Edit/delete/reply** same as footer.
- **Deep links**: `?focusedCommentId=` opens panel scrolled to thread; used by notification emails.

## Reactions (if present) `[V]`
Emoji reactions on page/comment with picker and tooltip of reactors.

## Accessibility
Presence changes announced politely; comment panel is a labelled complementary region; highlights are focusable marks with `aria-describedby` to thread.
