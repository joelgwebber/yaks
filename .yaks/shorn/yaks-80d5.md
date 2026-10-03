---
id: yaks-80d5
title: Map the coordination skills against an environment x farm-mode matrix
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:39:13Z'
parent: yaks-b5a0
labels:
- skills
---

Inventory what the yaks, yaks-coordinating, and yaks-working skills each assume, per cell of: environment {single agent, git worktrees + coordinator, Delta subagents, Delta human subthreads, PR-driven} x farm mode {team/public, private/local, out-of-tree pointer}. For each cell: is it documented, has it been run for real, what broke. Output: a matrix artifact plus the contradictions and gaps it exposes. It drives the restructure.

![skills-matrix](artifacts/yaks-80d5/skills-matrix.md)

---
▸ 2026-10-03T20:38:58Z [delta-lead]
Matrix of environment x farm mode with doc/run status per cell, mixed-axis assumptions per skill, 11 contradictions/gaps, and what they imply for 5c9f/2f9b/fe33. Cells marked P are predictions, not runs.

---
▸ 2026-10-03T20:39:13Z [delta-lead]
SHORN SUMMARY. Done: skills-matrix.md (5 environments x 3 farm modes, doc/run status per cell; per-skill assumptions; 11 contradictions/gaps). Learned: (1) most cells outside git-worktrees x team are undocumented and unrun; (2) private/out-of-tree + Delta has no isolation (walk-up reaches the user's live farm, reproduced in a /tmp tree, log O11); (3) mode forks are scattered across 4 sections of yaks-coordinating, which supports 5c9f; (4) the file-tool SOP and wt/ mechanics are git-worktree-only. Evidence: the attached matrix, and the reproduction recorded in yaks-df61 O11. Cells marked P are predictions, not runs; they are the work list for yaks-fe33. Spawned: none; candidate CLI change (discover stops at git top-level) is noted in O11, not yet a yak. Judge is the human reading the matrix: regrow if the framing is off.
