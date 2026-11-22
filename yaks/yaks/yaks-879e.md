---
id: yaks-879e
title: yaks commit errors 'git commit failed ... nothing to commit' when an autocommit already took the change
status: shaving
type: bug
priority: 3
created: '2026-11-22T00:15:23Z'
updated: '2026-11-22T16:40:42Z'
tags:
- git
- cli
---

Seen repeatedly on 2026-11-21/22 (herd-files): after a `yaks update`/`attach` that autocommitted, `yaks commit` sometimes prints `commit: nothing to commit under the farm` (right) and sometimes `Error: git commit failed (exit status: 1); the farm changes are staged, not committed: ... nothing to commit, working tree clean` with exit 1 (wrong: nothing was staged). Likely a race between its status check and the autocommit, or a stale index read. It should report nothing to commit, exit 0.

#### 2026-11-22T16:40:42Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
