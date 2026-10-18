---
id: yaks-9f5c
title: 'Optimistic concurrency on whole-body edits: TUI description/block edit must not clobber a concurrent note'
status: shaving
type: bug
priority: 2
created: '2026-10-18T05:15:36Z'
updated: '2026-10-18T20:00:00Z'
parent: yaks-6a34
labels:
- tui
- cli
---

`Farm::update` re-reads under the farm lock, so notes, moves and field edits are safe against concurrent writers; but `description = Some(body)` replaces the whole body with what the editor READ, so a TUI description edit (or `--description` on the CLI) saved after another process appended a note drops that note. Exists today within one shed; under the live-farm trial (yaks-6a34) every shed writes the same files, so it is likelier.

Fix: optimistic concurrency, not a CRDT. The TUI remembers the body (or `updated:`) it read; `Farm::update` with a body takes the expected base and, if the file moved on, three-way-merges the new body against the read base (the common case, "a note was appended below", merges trivially) or refuses and reloads. `blockedit.rs` already edits one block, so it can become a block-level patch. Call sites: handlers.rs (edit form), blockedit.rs, the CLI `--description`.

Evidence: a test that fails first (read, concurrent append, save: the note survives). Delegable.

---
▸ 2026-10-18T20:00:00Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T20:00:00Z [delta-farm]

Claim (delta-farm, yakherd; trial round 2 of yaks-6a34). Owner: `clobber-1`.

Goal: a whole-body save can never drop text another process wrote meanwhile. Today `App::commit_block` (src/tui/blockedit.rs) rebuilds the body from the App's in-memory `task.body`, possibly stale, and `Farm::update` writes `description: Some(body)` verbatim under the lock.

Design (decided, not yours to fork): optimistic concurrency in the library, one rule for every caller.
- `TaskEdit` gains `base: Option<String>`: the body the caller's edit was made against. `Farm::update` with a description and a base: if the file's current body == base, write as today. Else three-way: the common case (current = base + appended entries, i.e. a note or a move added after the read) writes `new + the appended suffix`; anything else is NOT written and returns a new `UpdateOutcome::Conflict` (nothing lost on disk; the caller keeps the user's text).
- TUI: `commit_block` passes the body it parsed as `base`; on `Conflict` it reloads and keeps the user's text recoverable (reopen the editor with it, or put it in a notification + the clipboard; your call, recorded as `Decision:`). Find any other whole-body save in src/tui (grep `description: Some`) and route it the same way.
- CLI `update --description` keeps today's semantics (no base: explicit overwrite), unchanged.

Scope: src/farm.rs (`TaskEdit`, `Farm::update`, `UpdateOutcome`), src/tui/blockedit.rs (`commit_block`), the conflict handling at its call site. Do NOT touch src/tui/editor.rs's `EditAction` enum (attach-edit-1 adds a variant there) or the attachment viewer code.

Evidence: tests that fail first: library (base read, concurrent `update --note`, save with base: both survive; a concurrent edit of the SAME text: Conflict, file unchanged); TUI (open block editor, another writer appends a note via `Farm::update`, save: the note survives). Gate `cargo test --workspace`. Judge: delta-farm.
