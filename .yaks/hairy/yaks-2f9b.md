---
id: yaks-2f9b
title: 'Delta skill: layers on top of the farm-mode coordination skills'
type: task
priority: 2
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T21:22:16Z'
parent: yaks-b5a0
depends_on:
- yaks-5c9f
labels:
- skills
- delta
---

Delta-specific guidance, starting from the case study: the parent thread is the coordinator and claims (shaves) before spawn_subagent; the worker task names the yak and its evidence contract; workers never commit path dependencies or other machine-local config; the coordinator re-runs verification before accepting a worker's 'tests pass'; one landing point (the coordinator pushes to local and/or origin, chosen in context); extra repos are attached as Delta worktrees, not nested clones; how to resync a parent left stale by landing elsewhere.

---
▸ 2026-10-03T20:39:31Z [delta-lead]
Requirements gathered from the observation log (yaks-df61): worker-brief template (O5: yak id, how to get the binary, actor name, scope, evidence contract + judge, forbidden moves, commit-or-leave, return format); export YAKS_ACTOR / --as (O4); build the binary in your own checkout (matrix gap 4); git add new artifact dirs (O9); coordinator re-runs verification and rejects what it cannot reproduce (O6); reviews happen after landing because finished workers' commits arrive in the parent (O2); skills are not loadable via the skill tool mid-thread, so inline the rules (O1); private/out-of-tree farms are shared live across threads (O11); extra repos are attached by the human (O10). Unverified, needs a trial: uncommitted claim visible to workers (O3), worker un-staged evidence on merge (O9b), push into the user's checkout (O12).

![worker-brief-template](artifacts/yaks-2f9b/worker-brief-template.md)

---
▸ 2026-10-03T21:14:01Z [delta-lead]
Draft worker-brief template (team-mode, Delta). Each section traces to an observation-log entry. First used in trial 1 (yaks-fe33); revise from what the workers do.

---
▸ 2026-10-03T21:14:11Z [delta-lead]
Correction to the requirements note above: project skills ARE loadable via the skill tool mid-thread (log O1 result) when they are real files and the worktree argument is passed; symlinked skill dirs are skipped. Inline the critical rules in worker briefs anyway: a worker's catalog depends on what its repo ships.

---
▸ 2026-10-03T21:22:16Z [delta-lead]
Brief template revised after trial 1 (v2): glob pathspec for git add, blocked-worker rule (no revert, no TMPDIR), docs-parity grep in the final message, and a new coordinator checklist (section 8) from O2/O22/O25. Same artifact, edited in place.
