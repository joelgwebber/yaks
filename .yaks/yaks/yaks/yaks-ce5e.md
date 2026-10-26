---
id: yaks-ce5e
title: Make answer comments use the full comment editor
status: shorn
type: task
priority: 3
created: '2026-10-19T04:27:59Z'
updated: '2026-10-26T21:06:03Z'
labels:
- tui
---

We just get a one-line editor right now, which makes it difficult to answer more complex questions, and edit as you go. It's going to become a comment anyway, so might as well use the same comment affordance as usual.

---
▸ 2026-10-26T20:55:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T20:55:58Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `answer-editor-1` (Delta subthread).

Goal: the TUI's ask and answer prompts (`a`, `App::open_ask` / `App::open_answer` in src/tui/handlers.rs) open the full multi-line comment editor (the `M` affordance, `Overlay::BlockEdit` via src/tui/blockedit.rs) instead of the one-line `Overlay::Edit` prompt. Saving still runs `needs_edit` (ask → needs: human; answer → needs: agent) with the typed text as the note; cancel/empty behave as today. Ask is included because it is the same prompt with the same problem.

Scope: `open_ask`, `open_answer`, `needs_edit`, the `EditAction::Ask`/`Answer` commit path in src/tui/handlers.rs, src/tui/blockedit.rs, src/tui/editor.rs. Help overlay (`help_content`) and docs/tui.md if the wording changes.

Evidence: a TUI test that fails before (a multi-line answer round-trips into the note with needs: agent), insta snapshot of the open answer editor, headless frame attached. Gate: `cargo test`. Judge: delta-lead at landing.

---
▸ 2026-10-26T20:56:27Z [answer-editor-1]

Started 2026-10-26T20:56:23Z (`date -u`: Mon Oct 26 20:56:23 UTC 2026).

- `yaks path yaks-ce5e`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-ce5e.md
- base: 2b1f3ba

[answer-editor-headless](yaks-ce5e/answer-editor-headless.txt)

[answer-editor-after-save](yaks-ce5e/answer-editor-after-save.txt)

---
▸ 2026-10-26T21:02:50Z [answer-editor-1]

Decision: ask and answer open `M`'s new-comment editor through a new `App::open_needs_edit(answer)` in `src/tui/blockedit.rs`, sharing a factored-out `open_slot_editor` with `open_block_edit`; the `EditAction::Ask`/`Answer` carried by the editor routes Ctrl-S to `needs_edit`. Rejected: a separate full-screen multi-line `Overlay::Edit` (a second editor UI, not "the same comment affordance"); a flag on `BlockEdit` (the action already says where the save goes).

---
▸ 2026-10-26T21:02:50Z [answer-editor-1]

Decision: the verb lives in the provisional header label (`▸ Answer <id> · <actor> · now`) and in the right-hand hints (`Ctrl-S answer (needs: agent) · Ctrl-C cancel`, shortening to `Ctrl-S answer`), plus the bottom status line (`Ctrl-S answer: needs agent · …`). Rejected: keeping the old one-line prompt text (`— hands it to an agent (Enter save …)`), since Enter is now a newline and the header has limited width.

---
▸ 2026-10-26T21:02:50Z [answer-editor-1]

Decision: empty/cancel behaviour kept as today: Ctrl-S on an empty editor still asks/answers with no note (as Enter on an empty one-line prompt did, unlike `M` which refuses an empty comment); cancel of an empty editor closes with no confirm; cancel with text now asks "discard?" first, like every multi-line editor. Rejected: making an empty save a no-op (would change the existing "answer without comment" path).

---
▸ 2026-10-26T21:02:50Z [answer-editor-1]

Decision: removed the old live tests `ask_raises_needs_and_records_the_question` / `answer_hands_the_yak_to_an_agent_and_records_the_reply` (they committed with Enter, now a newline) and replaced the yaks-027a label-separator test with `ask_and_answer_editors_name_the_verb_and_the_save`; the new multi-line live tests cover the same outcomes. After a save the line cursor goes to the new note, as after `M`. Rejected: keeping Enter-as-save tests by adding a special Enter binding (would break multi-line typing).

---
▸ 2026-10-26T21:03:55Z [answer-editor-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-26T21:04:04Z [answer-editor-1]

Summary: `a` (ask / answer) now opens `M`'s multi-line in-place comment editor instead of the one-line prompt.

- `src/tui/blockedit.rs`: new `open_needs_edit(answer)`; `open_block_edit` and it share `open_slot_editor` + `note_author`; header hints and status line name the verb and what Ctrl-S does (`hints_for`).
- `src/tui/handlers.rs`: `open_ask`/`open_answer` delegate; `EditAction::Ask`/`Answer` commit via `commit_needs` -> `needs_edit` (trimmed text as note, none if empty), then cursor to the new note.
- Tests (`src/tui/tests.rs`): `live::answer_editor_saves_a_multi_line_answer`, `live::ask_editor_saves_a_multi_line_question`, `live::answer_editor_empty_save_and_cancel`, `ask_and_answer_editors_name_the_verb_and_the_save`, insta `answer_editor_open`. All but the empty/cancel one FAIL with the source change reverted (checked: 5 failed, 1 passed), all pass with it.
- Real binary: headless frame of the open answer editor attached (`answer-editor-headless.txt`); after `key C-s` the throwaway yak shows `needs: agent` and a note `The disk one.\nIt survives restarts.` (`answer-editor-after-save.txt`).
- Docs: `docs/tui.md` key table row for `a` + a paragraph under "Editing in place". `help_content` ("Ask / answer (answer: needs agent)") is still accurate, left unchanged.
- Gate: `yaks verify yaks-ce5e` PASS.

---
▸ 2026-10-26T21:04:05Z [answer-editor-1]

moved: shaving -> shorn

---
▸ 2026-10-26T21:06:03Z [delta-lead]

Landed as 595697b (delta-lead, yakherd), via yaks land of worker commit 5d6a85e. Full gate re-run: 822 passed, 0 failed. Reproduced with my own debug build on a throwaway farm: a then two typed lines then Ctrl-S sets needs: agent, and the note holds both lines.
