---
id: yaks-000b
title: Mouse tweaks
status: shaving
type: task
priority: 3
created: '2026-11-03T21:04:56Z'
updated: '2026-11-07T17:38:37Z'
tags:
- tui
needs: human
---

A few scrolling odds and ends:
- If we can get horizontal scroll events from the mouse via terminals, it would be helpful to apply that to horizontally-scrollable areas in details (eg, overflowing tables).
- Vertical scrolling in list views moves the cursor, rather than the scrollable area. I think this would be better if it worked like the detail view.
- It would be nice if detail fields could be click-selected in edit mode.

---
▸ 2026-11-03T21:59:28Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T22:04:36Z [delta:starred-batch]

Two of three done in dab80e4:

- **Horizontal wheel:** crossterm reports `ScrollLeft`/`ScrollRight` (trackpad swipes, tilt wheels; whether a given terminal sends them is the terminal's call). Over the detail or an open page, it puts the line cursor on the row under the pointer and scrolls that code block or table sideways ([mouse.rs](code:src/tui/mouse.rs)).
- **List wheel:** now scrolls the list's viewport three rows a notch, like the detail; the cursor stays until it would leave the screen. With a yak's detail open, the list wheel still steps tasks (`J`/`K`), unchanged. An open page also scrolls on the wheel now (it had no hit rect, so the wheel did nothing over it).

Evidence: tests `wheel_scrolls_the_list_and_drags_the_cursor`, `horizontal_wheel_scrolls_the_block_under_the_pointer`; `?` Mouse section and `docs/tui.md` updated. Not tried with a real trackpad.

---
▸ 2026-11-03T22:04:36Z [delta:starred-batch]

kind: ask
needs: human
---

asked: needs human

One left: "detail fields click-selected in edit mode". Which edit mode do you mean?

1. **The create/edit form** (`E` on a yak: title, type, priority, tags, description rows). Lean: clicking a row focuses it (and a chip toggles), the same as Tab-ing there. Small.
2. **The in-place block editor** (editing a description or note in the pane). Lean: a click moves the editor's text cursor to that spot. This needs edtui to take a mouse position, which may mean a change in the edtui fork.

Or both? I'd start with 1.

---
▸ 2026-11-03T22:05:10Z [delta:starred-batch]

For option 2 of my ask: the edtui fork already has `events/mouse.rs` (`MouseEventHandler`, click-to-position and drag-select), so a click in the in-place editor may only need forwarding the event with the editor's rect. Smaller than I guessed.

---
▸ 2026-11-03T22:28:12Z [Joel Webber]

kind: answer
needs: agent
---

Oh, I totally missed the edtui case, but both of those would be useful. Feel free to add whatever you need to, to our edtui fork.

---
▸ 2026-11-07T17:04:27Z [herd-stars]
kind: pickup
---

picked up

herd-stars: both edit modes (form row click-focus, and click-to-position in the in-place editor) go to a worker.

---
▸ 2026-11-07T17:05:33Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-form`, after yaks-255d, in the same checkout, as a second commit.

**Scope.** Both edit modes, per Joel's answer:

1. **The create/edit form:** a left click on a row focuses it (as Tab-ing there); a click on a chip selects it.
2. **The in-place block editor:** a click inside it moves the text cursor there, by forwarding the mouse event to edtui's `MouseEventHandler` with the editor's rect (drag-select too, if edtui already does it).

- Files: `src/tui/mouse.rs`, `src/tui/create.rs`, `src/tui/editor.rs`, the `?` Mouse section (`EXTRAS` in `keymap.rs`), `docs/tui.md`.

**Out of scope.** Any change to the edtui fork: if forwarding is not enough, `yaks ask` with what edtui lacks.

**Evidence.** Tests driving mouse events at a form row and inside the editor (cursor position asserted). Frames attached. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-07T17:07:50Z [stars-form]

Start: Sat Nov  7 17:07:49 UTC 2026

- farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-000b.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/e3chjzn4e2pq/yaks

---
▸ 2026-11-07T17:32:29Z [stars-form]
kind: ask
needs: human
---

asked: needs human

Blocked on a one-line `Cargo.toml` change, which my brief forbids me to make. Question for the yakherd: may the root `Cargo.toml` enable edtui's `mouse-support` feature?

- edtui's `MouseEventHandler` (`src/events/mouse.rs`, checkout `c49afd9`) is behind `#[cfg(feature = "mouse-support")]` in `src/events/mod.rs`. The feature is in edtui's `default`, but our dependency line has `default-features = false, features = ["arboard"]`, so `edtui::events::MouseEventHandler` does not exist for us: `error[E0433]: cannot find MouseEventHandler in events ... gated behind the mouse-support feature`.
- The fix is `features = ["arboard", "mouse-support"]` on the `edtui = { git = ..., branch = "yaks-integration", default-features = false, ... }` line (line 28). The feature is empty (`mouse-support = []`, no new dependencies), so `Cargo.lock` does not change.
- Nothing is missing from edtui itself: its handler already does click-to-position (wrap-aware, tab-aware) and drag-select (Visual mode), so no edtui change is needed.
- Not changing it means writing our own click-to-position against edtui's private `LineWrapper` and `ViewState::screen_area` (both `pub(crate)`), which is not possible from outside.

Status: the form part (row focus, chip select, forwarding into a focused field) and the block-editor forwarding (position translated from the pane into the editor's scratch buffer) are written and uncommitted in my working tree; the tests are written and fail for the right reason on the old code. They cannot compile in this checkout until the feature is on. I will check the work in a scratch copy with the feature enabled (not committed, not a checkout of record) and report.

[frame-000b-block-editor-click](yaks-000b/frame-000b-block-editor-click.txt "Headless frame of the in-place editor after clicks (state asserted in a_click_in_the_block_editor_moves_the_text_cursor; a text frame draws no cursor)")
