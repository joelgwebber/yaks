---
id: yaks-6af1
title: 'sheds: a fork point that follows a branch switch (clone starts on main, then checks out the branch)'
status: shorn
type: bug
priority: 2
created: '2026-10-16T19:53:16Z'
updated: '2026-10-16T20:36:16Z'
parent: yaks-5662
labels:
- cli
- delta
---

`fork_point` takes the oldest entry of the shed's HEAD reflog. A Delta clone is created on whatever branch the source has checked out (on this machine Delta clones at `main`) and the thread then runs `git checkout <branch>`. The oldest entry is then the `main` commit, which is an ancestor of the branch HEAD, so it passes validation and the shed is credited with the whole branch.

Seen: rzzejj28kv0t (surveyor), reflog `c431937` (main) at 14:36, then `checkout: moving from main to kinds-discovery` to 8ef0612 at 14:39. It read `landed (39 yaks)` and showed main's claims as its own (`shaving: survey-209f, survey-3237, survey-9b8a · active`).

Proposed rule: the fork point is the NEW side of the most recent `checkout: moving from X to Y` entry in the HEAD reflog, when Y is the branch the shed is on now; else the oldest entry as today. Validation (an ancestor of HEAD) stays.

Failure modes to cover with tests and to judge in review:
- Switching back and forth (A -> B -> A): taking the latest checkout into the current branch drops work done on that branch before the round trip. Probably acceptable; say so.
- A shed that later checks out a NEW branch from its own work (`checkout -b feature` mid-arc) would lose the credit for its earlier commits. Candidate guard: only move the fork point when the checkout target is not a descendant of the shed's own earlier HEAD (switching to someone else's branch, not branching off your own).
- Detached HEAD (`moving from main to <sha>`) and `checkout -` produce the same message shape.
- `git switch` writes `checkout: moving from ...` too; `rebase`/`reset` do not and are left alone.
- Pre-existing reflog expiry is unchanged (no reflog: merge-base fallback).

Also check `pooling-refine-merge` (g4p5j2nkta4r, now gone): its oldest entry was 920e0f9 (main, Oct 13), then `reset: moving to origin/main` and `checkout: moving from main to kinds-discovery` to bd8658b, so the same bug credited it with ~18 extra commits. Its rebases later complicate it further; the rule only needs to beat today's answer there.

Evidence: tests for a clone that starts on main and checks out a branch, the round trip, and `checkout -b` from own work; docs (`sheds` text in docs/cli.md, README, skill, `Sheds` help) describe the new rule. Real machine: from surveyor, rzzejj28kv0t (if still around) or a fresh clone that checked out kinds-discovery reads only its own changes.

---
▸ 2026-10-16T20:31:43Z [delta]

moved: hairy -> shaving

---
▸ 2026-10-16T20:36:16Z [delta]

Decision: the fork point is the target of the LATEST `checkout: moving from X to <current branch>` entry in the HEAD reflog. It is skipped when that target already holds commits the shed made earlier: any HEAD-moving entry after the oldest that isn't a `checkout:` or `reset:` (Delta writes its commits with an empty message, so a `commit:` prefix can't be relied on). That covers `checkout -b` off its own work and round trips back to a branch it committed on. Otherwise it falls back to the oldest entry, as before. Ancestor-of-HEAD validation is unchanged. Rejected: the branch reflog (`logs/refs/heads/<b>`). A Delta clone's branch reflog starts at its first commit, not at the switch (rzzejj28kv0t: first entry `8ef0612 -> e7f6045`), so it would need the same guard anyway and adds a second file format. Parsing is pure (`fork_candidates`) and unit-tested.

---
▸ 2026-10-16T20:36:16Z [delta]

Evidence. New tests:
- `fork_candidates_follow_a_switch_onto_the_current_branch` (pure): surveyor-shaped reflog, another branch or detached HEAD, a round trip, empty-message commits, a null oldest entry.
- `a_shed_that_switched_branches_is_measured_from_the_switch` (real repos): switched onto `feat`, measured from the switch, `who` only the shed's own actor. Also `checkout -b` off its own commits keeps the original fork point. Disabling the switch made it FAIL (`left: 53c6627`, the oldest entry; `right: 490cceb`, the switch).

Full `cargo test --release`: green.

Real machine, from ~/src/fs/subtext/surveyor, the heuristics-models clone (rzzejj28kv0t: created on main at c431937, switched to kinds-discovery at 8ef0612):
```
before: who: delta, delta-sonnet, Joel Webber, delta:Review survey-e0c0 arc ... · shaving: survey-209f, survey-3237, survey-9b8a, survey-a0ce · active
        farm: 20 new, 20 moved, 132 new notes, 11 edited (30 landed)
after:  who: delta · shaving: survey-3237, survey-9b8a, survey-a0ce · active
        farm: 5 new, 4 moved, 36 new notes, 2 edited
```
survey-209f (main's claim) is gone from its row. The remainder is its own 19 commits (6 touching .yaks) plus uncommitted farm edits. Docs updated in docs/cli.md, README.md, the yaks skill, the `Sheds` help, and the module docs.

---
▸ 2026-10-16T20:36:16Z [delta]

moved: shaving -> shorn
