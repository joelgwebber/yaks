---
id: yaks-d262
title: 'TUI: optimistic saves, run the per-change commit after the saved frame is drawn'
status: hairy
type: task
priority: 3
created: '2026-11-04T22:31:40Z'
updated: '2026-11-05T05:33:24Z'
parent: yaks-f677
depends_on:
- yaks-7058,yaks-bcd4,yaks-1ebf
tags:
- tui
- perf
---

Deferred from yaks-907a (Joel: "optimistic concurrency in the TUI; file a yak and defer for re-measurement").

Show the saved text at once and run the per-change farm commit after that frame is drawn. The file is written but uncommitted for a moment, so it needs a rule for what can catch it in between: a quit, a crash, a `yaks sync` from another process, the watcher seeing a dirty farm. Flush on quit and before any `W`/`U`.

Do not start until yaks-7058, yaks-bcd4 and yaks-1ebf have landed and a changing save has been re-measured (expected ~200-250 ms). If it still feels slow, this is the next lever; otherwise shear it as not needed.
