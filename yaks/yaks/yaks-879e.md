---
id: yaks-879e
title: yaks commit errors 'git commit failed ... nothing to commit' when an autocommit already took the change
status: shaving
type: bug
priority: 3
created: '2026-11-22T00:15:23Z'
updated: '2026-11-22T17:15:04Z'
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

#### 2026-11-22T17:15:03Z

*commit-1*

Decision: two-part fix, both in the one rule "nothing to commit means nothing". (1) `yaks commit`, `yaks sync`'s commit step and the TUI `W` take the farm lock (new `store::lock_for_commit`, = `lock_uncommitted` on a farm branch, none elsewhere) around read+stage+commit, so no autocommit can interleave. (2) after `git add`, `git diff --cached --quiet -- .` is git's own verdict: clean -> `Outcome::Nothing` (covers a hand revert/no-op the status read missed, and any writer that doesn't take the lock, e.g. a person committing by hand). Rejected: only (2) (a first lock-free run still hit `git add` failing on index.lock held by the writer's `reset`: 5 of 150 runs errored); only (1) (leaves the hole for non-yaks committers); a retry loop on "nothing to commit" (string-matching git's text); catching the commit failure and re-reading status (extra git spawns in the error path, same race).

#### 2026-11-22T17:15:04Z

*commit-1*

Decision: `lock_for_commit` locks only when `farm_git(root)` says it is a farm branch; a plain `lock_uncommitted` on a YAKS_DIR/in-tree farm creates an untracked `.lock` the `.gitignore` never names, and broke `tui::tests::memo::w_on_a_clean_farm_says_so...` (the commit then saw `farm config`). Only a farm branch autocommits, so only there is there a writer to wait for. Rejected: always lock_uncommitted (that test failure); `store::lock` (it would autocommit on release and snapshot first).
Decision: in `yaks sync` the lock is held for the commit step only, in a block, because `pull` takes the (non-reentrant) lock itself. `--dry-run` takes no lock (it only reads). No git calls added to writes; the spawn-budget test passes.
Decision: test uses a `git` shim on PATH that makes the writer's commit land exactly when `yaks commit` runs `git add -A` (deterministic), rather than a timing loop. It exercises the post-add check, not the lock.

#### 2026-11-22T17:15:04Z

*commit-1*

Evidence: new test `commit_is_not_an_error_when_a_commit_took_the_change_meanwhile` (tests/autocommit.rs) FAILED before the fix with "Error: git commit failed (exit status: 1); ... nothing to commit, working tree clean", passes after. Scratch race loop after the final binary (60 writes vs 150 commits, scratch farm): fails=0, "nothing to commit under the farm" x150, errors=0 (before: 42 failures; after post-add check only: 5 errors, git add vs index.lock).
