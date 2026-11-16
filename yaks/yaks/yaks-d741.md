---
id: yaks-d741
title: 'Research: DeltaDB (Zed/Delta) as a store or sync layer for yaks'
status: shorn
type: task
priority: 3
created: '2026-11-16T21:32:33Z'
updated: '2026-11-16T21:34:48Z'
tags:
- research
- delta
- git
---

Joel asked: look at DeltaDB (https://delta.dev/deltadb) and anything usable outside Delta (especially open source); save the sources; write up what it could mean for storing/syncing a farm, as an alternative or parallel to commits on the yaks branch. Specifically: what wiring and server infrastructure it needs, and whether it is open source now or planned. Prior art: yaks-4fe6 (git-bug-style op-log in refs, dead: the farm stayed plain files on an orphan branch).

#### 2026-11-16T21:32:40Z

*delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA · kind: move · from: hairy · to: shaving · thread: ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-16T21:34:42Z

*delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA*

Findings in /doks/research/deltadb-for-yaks.md; 14 sources saved (Delta docs and blog, Zed blog, HN, Syntax transcript, delta-claude README). Short version: DeltaDB is closed and hosted-only (Cloudflare); the only outside door is the beta delta CLI (the Delta app headless + Zed login), which syncs whole worktrees and threads, no general primitives. Open-source pledge from Aug 2025, no code since. Mounting the farm worktree via delta would work mechanically but char-level text merge defeats the frontmatter merge driver (yaks-4a53/yaks-4c8c). Recommendation: no change; revisit if a crate, self-hostable server or primitives API appears.

#### 2026-11-16T21:34:48Z

*delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA · kind: move · from: shaving · to: shorn · thread: ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
