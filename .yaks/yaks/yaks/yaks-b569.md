---
id: yaks-b569
title: Skill yaks-scheduling and the edits to yaks, yaks-working, yaks-coordinating
status: shaving
type: task
priority: 2
created: '2026-11-04T22:31:57Z'
updated: '2026-11-04T23:41:39Z'
parent: yaks-7fa6
depends_on:
- yaks-5e8d
tags:
- skills
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). Prose and skills stay with the coordinator.

- New bundled skill `yaks-scheduling` (add it to `BUNDLED` in `src/skills.rs`): the tick ritual (run `yaks tick`, work what is woken for an agent, report); how to create a recurring yak; the **watcher contract** from the design dok.
- `yaks` skill: replace "Nothing runs by itself: cron, an agent harness or the TUI" with the blessed answer; add `yaks tick` to session start and the command table. One line each in `yaks-working` and `yaks-coordinating`.
- Cold-read the new skill with a cheap model, as AGENTS.md prescribes (only if Joel asks for subagents).
- Grep `docs/`, `README.md`, `.agents/skills/` and `src/` for the old wording.

---
▸ 2026-11-04T23:41:39Z [sched-1]
kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
