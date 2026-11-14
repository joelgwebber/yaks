---
id: yaks-bf1b
title: Tag completion affordance for `L` shortcut, and the filter UI.
status: shaving
type: feature
priority: 3
created: '2026-11-11T15:55:50Z'
updated: '2026-11-14T11:32:24Z'
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
