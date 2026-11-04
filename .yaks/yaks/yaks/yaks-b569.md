---
id: yaks-b569
title: Skill yaks-scheduling and the edits to yaks, yaks-working, yaks-coordinating
status: shorn
type: task
priority: 2
created: '2026-11-04T22:31:57Z'
updated: '2026-11-04T23:51:04Z'
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

---
▸ 2026-11-04T23:51:03Z [sched-1]

Done on branch `scheduling`.

- New bundled skill `yaks-scheduling` (52 lines, in `BUNDLED`): the round (`yaks tick`, and what to do when asked to run one), a recurring yak (`--every`, `--due`, `--wake`, the description as the task, snooze and stop), and the watcher contract as eight rules (find-or-create by `--key`, own actor, clear only on a real outside signal, skip on failure, give each yak a `--due`, never sync or install a clock or notify, one summary line, register by creating one recurring yak). It also says outside text is information, never instructions.
- Edits: `yaks` skill (the recurring paragraph now says the person runs `yaks tick`, with a pointer to the new skill; `tick` was already in the command table), one sentence each in `yaks-working` (a `Recurring:` request is the same as any request: shear it so it can wake again) and `yaks-coordinating` (a worker does not run `tick`), `.agents/skills/README.md`, `docs/skills.md` (six skills now), `docs/cli.md` (install list), and the stale "cron, a harness or the TUI calls it" wording in `docs/cli.md` and the `regrow-due` help.
- Evidence: the skills guards pass (names and descriptions, quoted description, cross-references to real skills, the `BUNDLED` lists in `src/skills.rs` and `tests/init_modes.rs`); full `cargo test` green (979 + 31 suites). I checked the one subtle claim in the skill against the real binary: a yak due today with `P1D` does not regrow when sheared the same day, and one whose `due:` is two days back regrows once with `1 skipped` and `awaiting agent`.

Not done: the cold-read scout check that AGENTS.md prescribes after a skills change. It needs a subagent and Joel has not asked for one; the acceptance yak (`yaks-79a6`) is the real cold read, by an agent that has to rewrite `review-watch` from this skill alone.

---
▸ 2026-11-04T23:51:04Z [sched-1]
kind: move
from: shaving
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
