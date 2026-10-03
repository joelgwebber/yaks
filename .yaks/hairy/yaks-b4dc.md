---
id: yaks-b4dc
title: 'discover: walk-up crosses the git top-level and finds another checkout''s farm (Delta private mode)'
type: bug
priority: 2
created: '2026-10-03T20:39:31Z'
updated: '2026-10-03T20:39:31Z'
parent: yaks-b5a0
labels:
- cli
- delta
---

Found in the Delta run (yaks-df61 O11). store::discover walks up the filesystem with no stop at the enclosing git top-level. Delta checkouts live at <repo>/.delta/worktrees/<id>/<repo>, so a checkout with no .yaks/ of its own (private/out-of-tree mode, or any branch predating the farm) silently reads and WRITES the user's live farm; reproduced in a /tmp tree. The same walk-up is what makes in-tree git worktrees work in private mode (yaks-213b), so the fix must keep that. Options: (a) stop at the nearest git top-level unless a .yaks pointer file or YAKS_DIR names the farm, making private-mode worktrees opt in; (b) keep walk-up but have create/write commands print the resolved farm path when it is outside the current git top-level; (c) leave it and document it. Decide with Joel; (b) is the smallest safe change. Evidence when done: a test that builds the nested layout and asserts the chosen behavior.
