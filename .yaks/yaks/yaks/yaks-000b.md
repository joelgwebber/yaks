---
id: yaks-000b
title: Mouse tweaks
status: shaving
type: task
priority: 3
created: '2026-11-03T21:04:56Z'
updated: '2026-11-06T22:48:57Z'
tags:
- tui
needs: agent
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
