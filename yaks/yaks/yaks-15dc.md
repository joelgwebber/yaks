---
id: yaks-15dc
title: Ensure all the internal link types work as expected
status: shaving
type: task
priority: 3
created: '2026-11-19T22:16:40Z'
updated: '2026-11-19T23:29:35Z'
tags:
- tui
---

Various types of links -- it looks like all work except for line numbers:
- [Link](code:src/tui/actions.rs) to source
- [Link](code:src/tui/actions.rs:10) to source:line
- [Link](code:src/tui/actions.rs@58eb9a6) to source@short-hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864) @hash
- [Link](code:src/tui/actions.rs@58eb9a6248e509e2e4cca2e158e1e97ca2c9a864:10) @hash:line

Though perhaps I have that syntax wrong?

#### 2026-11-19T23:26:57Z

*sched-1*

Let's also consider line ranges in links, like the Github UI supports. This would be helpful for selecting subsets of a file visually, in the peek UI, as well as providing narrowed context for humans and agents.

#### 2026-11-19T23:28:12Z

*sched-1*

Also, also -- Might be cool to have line-range links open a restricted view of a file, from which you can also navigate to the whole file.

#### 2026-11-19T23:29:35Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
