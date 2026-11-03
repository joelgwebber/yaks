---
id: yaks-907a
title: ':wq pauses for ~500-1000ms, but ctrl-s finishes quickly'
status: shaving
type: bug
priority: 3
created: '2026-11-03T12:22:51Z'
updated: '2026-11-03T21:50:57Z'
parent: yaks-f677
tags:
- tui
- perf
needs: human
---

Feels like there might be an intentional delay for some reason?

---
▸ 2026-11-03T21:40:35Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T21:42:04Z [delta:starred-batch]

Investigated (delta:starred-batch), not fixed.

- In code, `:wq`/`:w`/`:x` and Ctrl-S end in the same `commit_edit` ([fn run_command](code:src/tui/handlers.rs#L1682), [fn editor_key](code:src/tui/handlers.rs#L305)); there is no delay or timer on the `:` path.
- Timed headlessly on a clone of this farm (adding a comment): Ctrl-S frame-to-frame 553 ms, `:wq` 587 ms. Both are the per-change farm commit plus reload after the save, i.e. yaks-7058 (49 git calls per write), not anything `:wq`-specific.

So headless shows no difference; whatever Joel sees may be terminal-side.

---
▸ 2026-11-03T21:42:05Z [delta:starred-batch]
kind: ask
needs: human
---

asked: needs human

I can't reproduce a `:wq`-only pause: headlessly both `:wq` and Ctrl-S take ~0.55-0.6 s, all of it the per-change farm commit after the save (yaks-7058). Two questions:

1. In your terminal, does Ctrl-S really feel instant on the *same* kind of edit (a comment or description, not a single-line field)? If both pause, I'd fold this into yaks-7058 (make the commit cheaper, or run it after the frame is drawn).
2. If only `:wq` pauses: which terminal, and is it inside tmux/zellij? An escape-time delay on the `Esc` you press before `:` is my next suspect.

---
▸ 2026-11-03T21:50:57Z [Joel Webber]

test comment
