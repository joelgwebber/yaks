---
id: yaks-a004
title: Unify "peek" across yaks + doks
status: shaving
type: feature
priority: 3
created: '2026-11-06T21:26:35Z'
updated: '2026-11-07T17:07:28Z'
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
