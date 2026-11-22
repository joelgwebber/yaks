---
id: yaks-879e
title: yaks commit errors 'git commit failed ... nothing to commit' when an autocommit already took the change
status: shaving
type: bug
priority: 3
created: '2026-11-22T00:15:23Z'
updated: '2026-11-22T17:15:02Z'
tags:
- git
- cli
---

Seen repeatedly on 2026-11-21/22 (herd-files): after a `yaks update`/`attach` that autocommitted, `yaks commit` sometimes prints `commit: nothing to commit under the farm` (right) and sometimes `Error: git commit failed (exit status: 1); the farm changes are staged, not committed: ... nothing to commit, working tree clean` with exit 1 (wrong: nothing was staged). Likely a race between its status check and the autocommit, or a stale index read. It should report nothing to commit, exit 0.

#### 2026-11-22T16:40:42Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T16:41:02Z

*herd-files*

Claim (herd-files): worker commit-1 (Sonnet), base 6ae40a60, batch before v0.0.41. Scope: src/commit.rs, src/changes.rs (read), tests. Judge: herd-files at landing.

#### 2026-11-22T17:15:02Z

*commit-1*

Start 2026-11-22T16:42:20Z. Checkout /Users/joel/src/yaks/.delta/worktrees/bak75rh27hds/yaks. Farm file: ~/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-879e.md

Reproduced (scratch repo, release binary before the fix). Hypotheses 1 (no-op rewrite/touch) and LFS (lfs enabled, PNG attached) do NOT reproduce: 3x `yaks commit` each -> "nothing to commit under the farm", exit 0. Real cause is hypothesis 2, a race: `yaks commit` took NO farm lock, so a concurrent writer's per-change commit (autocommit on lock release) could land between `changes::read` and `git add`/`git commit`. Run: a loop of 60 `yaks update --note` writes against 150 concurrent `yaks commit`: 42 exit 1, e.g. "Error: git commit failed (exit status: 1); the farm changes are staged, not committed: On branch yaks / nothing to commit, working tree clean".
