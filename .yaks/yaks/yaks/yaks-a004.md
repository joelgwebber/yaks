---
id: yaks-a004
title: Unify "peek" across yaks + doks
status: shorn
type: feature
priority: 3
created: '2026-11-06T21:26:35Z'
updated: '2026-11-07T17:25:49Z'
tags:
- tui
---

The "peek" affordance in doks can be really useful, but:
1. It can't be shown/hidden when needed.
2. We don't get it for yaks, just doks.

---
▸ 2026-11-07T17:04:42Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:05:34Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-peek`.

**Scope.** One peek, the same in the yak detail pane and the text viewer:

- The yak detail pane gets the viewer's peek pane (`render_peek`, `PEEK_ROWS`) when its line cursor is on a peekable link, in place of today's one-line status-bar peek.
- `p` (`peek.toggle`) shows and hides it in both, one flag (`peek_shown`) for the whole session.
- Check that `p` works in the viewer as opened from the Doks tab; fix it if not (Joel says it could not be hidden).
- `p:peek` in both contexts' bottom hint, so it is discoverable.
- Files: `src/tui/follow.rs`, `src/tui/render.rs` (detail render, `render_peek`), the `Detail`/`Viewer` rows in `src/tui/keymap.rs`, `docs/tui.md`.

**Out of scope.** What `links::peek` returns; the other keys.

**Evidence.** Tests: the yak pane shows the peek pane on a dok link, `p` hides it there and in the viewer. Frames of the yak pane with the peek, attached. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-07T17:07:28Z [stars-peek]

Start: 2026-11-07T17:07Z (`date -u`).

- `yaks path yaks-a004`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-a004.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/brgsw94dabng/yaks

---
▸ 2026-11-07T17:09:41Z [stars-peek]

Finding: `p` already works in the viewer from both entry points; I could not reproduce "can't hide".

- Wrote two tests in `src/tui/follow.rs` that drive real keys: open the hub dok from the Doks tab (`set_view` Doks, `j` to the row, Enter) and from a link in a yak's body (`l`, Tab, Enter). In each, Tab to a dok link, assert the `── peek:` rule is drawn, `p` removes it, `p` again restores it. Both pass on the unchanged tree.
- So the viewer's `p` (`Cmd::PeekToggle` -> `run_viewer_cmd`) is fine; what is missing is the toggle in the yak detail pane (whose one-line status-bar peek has no `p`), and discoverability (viewer `p` has `NoHint`; it is only in `?`). That matches the task's other items; keeping the two tests as regressions.
- One wart noted, not changed: `p` in a viewer on a dok with no peekable link flips the flag with a notification but nothing visible changes.

[yak-peek-frame](yaks-a004/yak-peek-frame.txt "Headless text frame (100x24) of the yak detail pane, cursor on a dok link: the peek pane under the lines, status bar shows the destination")

![tui-yak-peek](yaks-a004/tui-yak-peek.svg "Color SVG frame of the same scene (docshot tui-yak-peek)")

---
▸ 2026-11-07T17:25:20Z [stars-peek]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-07T17:25:48Z [stars-peek]

Decision: the detail pane reserves the peek rows only while it is focused (`App::detail_has_peek`: shown, focus Detail, no dok open, a farm, no block editor, a peekable link anywhere in the yak). Rejected: reserving them whenever the pane is drawn beside the list (7 rows lost, and a blank rule, for every yak with a link while the cursor is on the list) and showing the peek only when the cursor is on a link (the page would jump as the cursor moves; the viewer reserves for the same reason).

Decision: peekable means `Target::as_link().is_some()` (farm dok, source file / `code:` range, attachment), the same set `links::peek` reads, so an inline-image attachment link counts. One existing test (`an_inline_image_is_drawn_clipped_at_the_pane_edge_and_scrolls`, a 14-row pane) now sets `peek_shown = false`, since it measures image clipping in a pane too short to share.

Decision: the status bar's one-line peek (`detail_peek_line`) is removed, not kept as the fallback while the pane is hidden: `p` hides the peek, and the bar falls back to `→ <destination>` as for any other link. Rejected: a fallback line when hidden (two peeks, which is what the yak asked to unify).

Decision: `detail_has_peek` takes no rows and tests the cheap conditions first. `edit_slot` asks `detail_viewport` while the detail rows are being built, so a viewport that asked for the rows first recursed (stack overflow in `comment_overlay_paints_no_black_cells`). The pane is also off while the in-place block editor is open (it wants the rows).

Decision: `p` in Detail is `bind_pane(Yak, ...)`, so a shed's pane (no links) has no `p` and its hint is unchanged. The viewer's bottom hint is a hand-written string in `render_status`, not generated from the table (`hint_for(Ctx::Viewer)` is asserted empty), so I changed its existing `p peek` to `p:peek` there and left the Viewer row `NoHint`; adding a table hint would be dead.

Decision: toggling keeps the cursor in view (`toggle_peek`): the detail scrolls the cursor line into the new page, the viewer re-reveals its cursor (restoring `nav.link`, which `goto` clears). Without it, showing the pane could cover the cursor.

Decision (scope): I touched files beyond the named list because the work needs them, each by a few lines: `src/tui/detail_nav.rs` (`detail_viewport` subtracts the peek rows), `src/tui/handlers.rs` (`run_viewer_cmd` calls the shared `toggle_peek`; the old inline toggle is gone), `src/tui/keymap.rs` `run_cmd` (the `PeekToggle` arm, with the hint-test strings), `src/tui/tests.rs` (the one `peek_shown = false` line) and `src/tui/docshots.rs` (a new `docshots_yak_peek` scene). Not asked first because the geometry cannot work without `detail_viewport`; flagging it for the judge.

Decision: ran `cargo test -p yaks --bin yaks docshots -- --ignored`, which rewrote ~27 SVGs and `session*` with unrelated drift (tab names, opacity attributes); reverted all but the two this change explains: `tui-doks-dok.svg` (the viewer hint `p peek` -> `p:peek`) and the new `tui-yak-peek.svg`. `tui-doks-peek.svg` is unchanged (its frame has a link hint, not the key hint).

Summary: the yak detail pane now shows the viewer's peek pane (one `render_peek`, called from `render_attachment` and `render_detail`, `PEEK_ROWS`) when focused with a peekable link in the yak; `p` toggles `peek_shown` in both contexts (`App::toggle_peek`, `Cmd::PeekToggle` in the Detail yak rows and the Viewer row); `p:peek` is in both bottom hints; docs/tui.md updated (detail key paragraph, links paragraph, Peek bullet, new figure). Tests added in `follow.rs`: the yak pane shows the pane on a dok link and the status bar has only the destination; `p` hides/shows it in the yak pane, in a dok opened from it (one flag) and back; the focused-only and page-size rules; showing the pane scrolls a covered cursor into view; `p` hides/shows in a dok opened from the Doks tab and from a yak link (the two entry points the yak named). Insta snapshots changed only by the hint (`p:peek` in six Detail frames, `p:peek` in the Doks viewer frame) and one new help-overlay row; I accepted those.

Evidence not gathered: I did not run the new yak-pane tests against the unchanged tree (they cannot compile-pass without the new code: `p` was unbound in Detail and the status bar carried the peek); the viewer tests passed before and after, as found above.

---
▸ 2026-11-07T17:25:49Z [stars-peek]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEp9bZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
