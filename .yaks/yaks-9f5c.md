---
id: yaks-9f5c
title: 'Optimistic concurrency on whole-body edits: TUI description/block edit must not clobber a concurrent note'
status: hairy
type: bug
priority: 2
created: '2026-10-18T05:15:36Z'
updated: '2026-10-18T05:15:36Z'
parent: yaks-6a34
labels:
- tui
- cli
---

`Farm::update` re-reads under the farm lock, so notes, moves and field edits are safe against concurrent writers; but `description = Some(body)` replaces the whole body with what the editor READ, so a TUI description edit (or `--description` on the CLI) saved after another process appended a note drops that note. Exists today within one shed; under the live-farm trial (yaks-6a34) every shed writes the same files, so it is likelier.

Fix: optimistic concurrency, not a CRDT. The TUI remembers the body (or `updated:`) it read; `Farm::update` with a body takes the expected base and, if the file moved on, three-way-merges the new body against the read base (the common case, "a note was appended below", merges trivially) or refuses and reloads. `blockedit.rs` already edits one block, so it can become a block-level patch. Call sites: handlers.rs (edit form), blockedit.rs, the CLI `--description`.

Evidence: a test that fails first (read, concurrent append, save: the note survives). Delegable.
