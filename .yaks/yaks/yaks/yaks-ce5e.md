---
id: yaks-ce5e
title: Make answer comments use the full comment editor
status: shaving
type: task
priority: 3
created: '2026-10-19T04:27:59Z'
updated: '2026-10-26T20:56:27Z'
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
