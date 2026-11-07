---
id: yaks-a004
title: Unify "peek" across yaks + doks
status: shaving
type: feature
priority: 3
created: '2026-11-06T21:26:35Z'
updated: '2026-11-07T17:25:20Z'
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
