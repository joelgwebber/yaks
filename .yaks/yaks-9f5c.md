---
id: yaks-9f5c
title: 'Optimistic concurrency on whole-body edits: TUI description/block edit must not clobber a concurrent note'
status: shorn
type: bug
priority: 2
created: '2026-10-18T05:15:36Z'
updated: '2026-10-18T20:10:11Z'
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

---
▸ 2026-10-18T20:00:47Z [clobber-1]

Started (clobber-1): Sun Oct 18 08:00:42 PM UTC 2026. yaks path yaks-9f5c: /home/joel/src/yaks-farm/.yaks/yaks-9f5c.md

[clobber1-conflict-frame](yaks-9f5c/clobber1-conflict-frame.txt)

---
▸ 2026-10-18T20:07:58Z [clobber-1]

Decision: the three-way rule is "current == base (trailing whitespace ignored) → write new; current == base.trim_end() + `\n\n` + text starting with an entry header (`---\n▸ `) → write new + that suffix; else Conflict" (`farm::rebase_description`). Rejected: a general line-level diff3 (needs a dependency or a hand-rolled merge, and the claim note decided "the common case or refuse"); keying off `updated:` (a field-only edit such as a label change would then conflict needlessly, and it can't recover the appended entries).

---
▸ 2026-10-18T20:07:58Z [clobber-1]

Decision: on Conflict the TUI reopens the block editor on the same block of the reloaded body, holding the user's text, with `original` = the fresh block text (so it is dirty and a cancel still asks), notification `<id> changed on disk; not saved: review and Ctrl-S again`. A second Ctrl-S saves against the fresh body. If the block no longer exists, the text goes to the clipboard (`…, your text is on the clipboard`). Rejected: notification + clipboard only (the user loses the editing context and the clipboard is best-effort, unavailable headless); auto-merging the block anyway (would silently override the other writer).

---
▸ 2026-10-18T20:07:58Z [clobber-1]

Decision: `App::apply_edit` (src/tui/handlers.rs) now returns `Option<UpdateOutcome>` and handles `Conflict` itself (reload + `<id> changed on disk; not saved`); `commit_block` overrides that message when it reopens. Rejected: a separate conflict-only update path in blockedit.rs, which would duplicate apply_edit's reload/drop-out logic. `UpdateOutcome` gained `Debug, Clone, Copy, PartialEq, Eq` for the comparisons. CLI `update_one` (src/main.rs) maps Conflict to an error + non-zero exit; it's unreachable since the CLI always passes `base: None` (explicit overwrite, unchanged). Touching main.rs and handlers.rs was required by the exhaustive matches / the "conflict handling where apply_edit reports outcomes" scope.

---
▸ 2026-10-18T20:07:58Z [clobber-1]

Evidence:

- Tests (all fail with `Farm::update` ignoring `base`, i.e. pre-fix behaviour, 4/4 FAILED; pass with the fix): `farm::tests::description_with_base_keeps_a_concurrent_note`, `farm::tests::description_with_base_conflicts_on_a_concurrent_rewrite` (asserts the file bytes are unchanged), `farm::tests::rebase_description_cases`, `tui::tests::block_edit::saving_keeps_a_note_appended_meanwhile`, `tui::tests::block_edit::a_conflicting_save_writes_nothing_and_keeps_the_text`.
- Real run with `./target/release/yaks` in a temp farm: `create --description "old text"`, `update --note "a note"`, `update --description "cli overwrite"` → exit 0, body exactly `cli overwrite` (CLI explicit overwrite unchanged).
- Conflict frame attached: `clobber1-conflict-frame.txt` (90x30 headless draw: the other writer's `their rewrite`, the editor reopened holding `- beta!`, the notification).
- Only whole-body save in src/tui is `commit_block` (grep `description:`; handlers.rs:323 is the create form's NewTask).
- No insta snapshot changed. docs/tui.md: one paragraph addition beside "Editing in place".

---
▸ 2026-10-18T20:08:03Z [clobber-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-18T20:08:09Z [clobber-1]

Summary (clobber-1): `TaskEdit.base` + `UpdateOutcome::Conflict` in src/farm.rs (`rebase_description` keeps entries appended after the base, refuses anything else without writing); the TUI block editor's `commit_block` passes its base and on Conflict reopens the editor with the user's text over the reloaded yak; `apply_edit` returns the outcome and reports conflicts; the CLI keeps explicit overwrite (`base: None`). docs/tui.md describes the conflict. Gate `cargo test -p yaks` PASS via `yaks verify`. Files: src/farm.rs, src/main.rs, src/tui/blockedit.rs, src/tui/handlers.rs, src/tui/tests.rs, docs/tui.md.

---
▸ 2026-10-18T20:08:09Z [clobber-1]

moved: shaving -> shorn

---
▸ 2026-10-18T20:10:11Z [delta-farm]

Landed (delta-farm): worker commit 77f3517 SQUASHED onto main as c65dfe8 by land.sh (first real squash landing: one single-parent commit, worker author/message kept, Squashed-from trailer, tree identical to the computed merge, doctor clear). Full `cargo test --release` 865 passed. Real race, driven headlessly: open the description block editor, another process appends a note via the CLI, type, Ctrl-S. Previous ~/go/bin build: the note is LOST. This build: the description is saved and the concurrent note is kept after it. Accepted the out-of-scope compiler-forced edit to src/main.rs.
