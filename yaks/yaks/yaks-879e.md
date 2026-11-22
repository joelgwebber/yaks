---
id: yaks-879e
title: yaks commit errors 'git commit failed ... nothing to commit' when an autocommit already took the change
status: hairy
type: bug
priority: 3
created: '2026-11-22T00:15:23Z'
updated: '2026-11-22T00:15:23Z'
tags:
- git
- cli
---

Seen repeatedly on 2026-11-21/22 (herd-files): after a `yaks update`/`attach` that autocommitted, `yaks commit` sometimes prints `commit: nothing to commit under the farm` (right) and sometimes `Error: git commit failed (exit status: 1); the farm changes are staged, not committed: ... nothing to commit, working tree clean` with exit 1 (wrong: nothing was staged). Likely a race between its status check and the autocommit, or a stale index read. It should report nothing to commit, exit 0.
