---
id: yaks-1a16
title: Keyboard tweaks for tag auto-completion
status: shaving
type: task
priority: 3
created: '2026-11-11T20:23:06Z'
updated: '2026-11-14T13:25:26Z'
tags:
- tui
---

Currently, the tag autocomplete affordance eats `<tab>` right after you select a tag. Looks like this is because it leaves the cursor right after the tag just added, so `<tab>` pushes you back into the completion overlay.

"Fixing" this may be as simple as just adding a space after the selected tag, so that you can tab to the next field.

---
▸ 2026-11-14T11:32:22Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T13:25:15Z [tags-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-14T13:25:26Z [tags-1]

Fixed together with yaks-bf1b (same code): `insert_tag` in `src/tui/editor.rs` leaves the cursor after `, ` (or steps over a following separator) so Tab after a pick moves to the next field/row, not the picker. Applies to the form, the `L` editor and the drawer. Test: `tab_after_a_picked_tag_moves_to_the_next_field` (pick, then Tab: form row 3 -> 4, no picker), plus the drawer variant. Gate: `cargo test -p yaks` green, fmt clean, `yaks verify yaks-1a16` PASS. See yaks-bf1b notes for decisions and frames.
