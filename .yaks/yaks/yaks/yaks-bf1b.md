---
id: yaks-bf1b
title: Tag completion affordance for `L` shortcut, and the filter UI.
status: shaving
type: feature
priority: 3
created: '2026-11-11T15:55:50Z'
updated: '2026-11-14T13:18:23Z'
tags:
- tui
---

Reuse the tag completion overlay from the detail editor.

---
▸ 2026-11-14T11:32:22Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T11:32:24Z [linkplan-lead]

Claim: worker `tags-1` (Sonnet tier), owning this yak and yaks-1a16 together (both touch `finish_insert`, `tag_completion`, `open_ref_picker` and `docs/tui.md`). Sketch is in the triage note ([yaks-20dd](yaks-20dd#2026-11-11T22:08:04Z)). Owns: `tui/handlers.rs` (`editor_key` for the `L` editor, the filter drawer's tags row, `finish_insert`), `tui/editor.rs` (`tag_completion`), the hint in `tui/render.rs`, `docs/tui.md`. Out of scope: any key binding change (yaks-8874 is awaiting a decision), `keymap.rs`, link code. Evidence: unit tests plus a toque frame of the picker open over the filter drawer, attached.

---
▸ 2026-11-14T11:32:53Z [tags-1]

Start: Sat Nov 14 11:32:49 UTC 2026
Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-bf1b.md
Checkout: /Users/joel/src/yaks/.delta/worktrees/zkbr0g3vc11c/yaks
Also owns yaks-1a16 (same code).

---
▸ 2026-11-14T13:17:18Z [tags-1]

Milestone: code + docs done, tests passing (tag filter). Remaining: render toque frames, full gate, notes, commit.

- `insert_tag` (editor.rs): a pick replaces the partial tag and leaves the cursor after `, ` (or steps over a following comma/space) — fixes yaks-1a16.
- `editor_key`: the `L` editor (`EditAction::Labels`) uses `tag_completion` on Tab.
- Drawer row 3: Tab on a partial tag opens the picker; `finish_insert` has a Drawer arm and re-runs `drawer_live_preview`.
- Hint: `Tab → complete tag` on the drawer's tags row and beside the `L` line.

![drawer-hint](yaks-bf1b/drawer-hint.png "Filter drawer, tags row on a partial tag 'ru': the hint line says Tab → complete tag")

![drawer](yaks-bf1b/drawer.png "Tab in the drawer's tags row: the tag picker open over the filter drawer, seeded with 'ru'")

![l-hint](yaks-bf1b/l-hint.png "The L editor on a partial tag: 'Tab → complete tag' beside the line")

![l-picker](yaks-bf1b/l-picker.png "Tab in the L editor: the tag picker open over it, seeded with 'ru'")
