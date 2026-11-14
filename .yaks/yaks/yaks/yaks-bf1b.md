---
id: yaks-bf1b
title: Tag completion affordance for `L` shortcut, and the filter UI.
status: shorn
type: feature
priority: 3
created: '2026-11-11T15:55:50Z'
updated: '2026-11-14T13:25:26Z'
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

---
▸ 2026-11-14T13:21:01Z [tags-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-14T13:25:25Z [tags-1]

Decision: a pick always ends with the cursor past a separator. `insert_tag` adds `, ` after the tag, or steps over a comma/space that already follows. Rejected: adding only a space (the field would then hold `a b`, which splits fine but reads inconsistently with the `, ` the form writes) and a trailing comma without a space.

Decision: in the `L` editor Tab with nothing to complete falls through to edtui as before; no "next field" exists there. Rejected: swallowing Tab.

Decision: the `L` editor's Tab uses `tag_completion` instead of `completion_context`, so a `yaks-…` tag in `L` is a tag, not a yak ref (the field holds tags, not markdown).

Decision: the `L` hint is drawn at the right edge of the status line beside the line editor (only when there is room), because the single-line editor owns the whole status line. Rejected: changing `render_line_editor`'s layout for every editor.

Summary: tag completion now works in the `L` editor and the filter drawer's tags row, sharing `tag_completion`/the picker with the form. A pick goes through `insert_tag` (editor.rs), which leaves the cursor after `, ` so Tab moves on (yaks-1a16). `finish_insert` has `Overlay::Drawer` and tag `Overlay::Edit` arms; a drawer pick re-runs `drawer_live_preview`. Hint `Tab → complete tag` on the drawer's tags row and beside the `L` line. `docs/tui.md` updated (Completing a tag, tag fields paragraph).

Evidence: unit tests `tab_after_a_picked_tag_moves_to_the_next_field`, `picking_a_tag_before_a_separator_steps_over_it_instead_of_adding_one`, `tab_in_the_l_editor_completes_a_tag_and_a_pick_moves_past_it`, `tab_in_the_drawer_tags_row_completes_and_the_pick_reruns_the_filter`, `tab_on_a_complete_drawer_tags_row_still_moves_to_the_next_row`; the existing partial-tag and replace tests updated for the trailing `, `. Frames attached (drawer and `L`, hint and picker). Gate: `cargo test -p yaks` green, `cargo fmt --check` clean, `yaks verify` PASS.

---
▸ 2026-11-14T13:25:26Z [tags-1]
kind: move
from: shaving
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAGEdQZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
