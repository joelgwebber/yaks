---
id: yaks-000b
title: Mouse tweaks
status: shaving
type: task
priority: 3
created: '2026-11-03T21:04:56Z'
updated: '2026-11-07T17:39:08Z'
tags:
- tui
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

[frame-000b-form-chip-click](yaks-000b/frame-000b-form-chip-click.txt "Edit form after clicking the bug chip, p1 and the wakes label: focus is on the wakes row (the frame is plain text, chip selection is asserted in a_click_on_a_chip_selects_it)")

---
▸ 2026-11-07T17:38:50Z [stars-form]

Status: implemented in the working tree, NOT committed, NOT shorn: it cannot compile in this checkout until the question above is answered (`Cargo.toml` must enable edtui's `mouse-support` feature; I may not edit it).

Fail-first: the five new tests in `src/tui/mouse.rs` failed on the old code with the mouse ignored under an overlay: `a_click_on_a_form_row_focuses_it`, `a_click_on_a_chip_selects_it`, `a_click_in_the_focused_form_field_moves_its_text_cursor`, `a_click_in_the_block_editor_moves_the_text_cursor`, `a_drag_in_the_block_editor_selects` (the guard test `a_click_outside_the_block_editor_text_leaves_the_cursor` passes both ways, as it should).

Evidence, with the feature enabled in a scratch copy of this checkout in `/tmp/sf/copy` (the copy's `Cargo.toml` has `features = ["arboard", "mouse-support"]`; nothing from it is committed):

- `cargo test -p yaks mouse::tests`: 14 passed (all 7 new, 7 old).
- `cargo test --workspace --no-fail-fast` there: lib 1026 passed, 1 failed (`sources::tests::a_scratch_path_is_not_provenance_but_another_path_is`, which reads the checkout's own path and fails under `/tmp`); the CLI integration tests fail there too because the copy has no `.git`/farm. Neither is about this change; the real gate has to be run in the checkout once the feature is on.

Decision: the in-place editor is drawn into a scratch buffer, so edtui's `screen_area` is in scratch coordinates. The render records `EditorHit { clip, top }` (the pane rect and the screen row of the scratch buffer's row 0, negative when scrolled partly off) and the click is translated into scratch coordinates before it is forwarded to `MouseEventHandler`. Rejected: rendering the editor straight onto the frame (a half-scrolled-off editor would draw wrong; that is why it is a scratch buffer).

Decision: only the left button goes to edtui in the editor (Down, Up, Drag); the wheel is left as it was (ignored under an overlay). A drag puts the editor in its visual mode: that is edtui's drag-select, taken as it is.

Decision: in the form, the click that focuses a row only focuses it; a click in an already-focused text row or the description is forwarded and positions the text cursor. edtui learns where a field is from its last render, and only the focused row is rendered by edtui, so the first click has no screen area to resolve against. Rejected: an off-screen render pass of the clicked row's editor to place the cursor on the first click (a second code path for the view settings, for one click saved).

Decision: chip rects come from `render_chip_row` itself (it now returns them), so the hit-test cannot drift from the drawing; the form's row rects are recorded by `render_create` into `HitMap`.

Docs parity: `docs/tui.md` Mouse paragraph (the "ignored while an overlay is open" sentence is replaced: form and in-place editor take clicks, other overlays still ignore the mouse); two new rows ("Click form", "Click editor") in the `?` Mouse section (`EXTRAS`, `src/tui/keymap.rs`). No other file said the mouse is ignored under overlays.

---
▸ 2026-11-07T17:38:59Z [herd-stars]
kind: answer
needs: agent
---

Yakherd herd-stars: yes, enable it: `features = ["arboard", "mouse-support"]` on the edtui line of the root `Cargo.toml` (an empty feature of a dependency we already have, so `Cargo.lock` should not change; if it does, say so). Commit it with yaks-000b. Run the real gate in your checkout.

---
▸ 2026-11-07T17:39:08Z [stars-form]
kind: pickup
---

picked up
