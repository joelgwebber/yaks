---
id: yaks-6af1
title: 'sheds: a fork point that follows a branch switch (clone starts on main, then checks out the branch)'
status: hairy
type: bug
priority: 2
created: '2026-10-16T19:53:16Z'
updated: '2026-10-16T19:53:16Z'
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
