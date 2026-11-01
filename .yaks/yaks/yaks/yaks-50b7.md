---
id: yaks-50b7
title: 'Flaky test: init_modes uses a fixed temp path for its worktree'
status: hairy
type: bug
priority: 3
created: '2026-11-01T16:08:19Z'
updated: '2026-11-01T16:08:19Z'
labels:
- test
---

`tests/init_modes.rs` `farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo` adds a worktree at a fixed temp path (`$TMPDIR/share2-web-wt`). A second run in the same temp dir fails with `fatal: ... already exists`. Seen 2026-11-01 running the suite repeatedly in one shell; CI starts clean, so it passes there. Fix: include the process id in the path (like the other tests), or remove it first.
