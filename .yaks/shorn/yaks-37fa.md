---
id: yaks-37fa
title: 'yaks lanes: measure a lane''s own changes from where the lane forked, not from the viewer''s merge-base'
type: bug
priority: 2
created: '2026-10-06T21:05:06Z'
updated: '2026-10-06T21:41:51Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Found by Joel running `yaks lanes` from ~/src/yaks (2026-10-06): twelve lanes, every one labelled `who: delta-lead` plus a mixture of the others (one lane lists six actors), the same `shaving:` yaks and the same `farm:` changes repeated across lanes. He asked whether threads share worktrees. They do not: it is a defect in how "a lane's own changes" is defined (yaks-d738 / yaks-c29a).

## Cause
A lane's own changes are measured from the merge-base with the VIEWER's HEAD. A worker forks from the coordinator's thread, not from the viewer, so its history contains everything the coordinator committed before the fork; relative to the viewer all of that becomes "this lane's" work. Measured on the real lanes (commits since the lane's actual fork vs commits attributed to it):

- inp-1's lane: 1 vs 3x; agt-1's lane: 1 vs 23; ntf-1's lane: 1 vs 3x; the coordinator's own lane: 49 vs 38.

A second effect compounds it: after a squash landing the workers' original commits are no longer ancestors of the viewer's HEAD, so a finished, landed lane still looks unmerged.

## Fix
Measure a lane from where IT forked. The fork point is the oldest entry of the lane's HEAD reflog (`<git-dir>/logs/HEAD`, first line), which exists for a Delta clone and for a `git worktree add -b` lane (core.logallrefupdates). Use it as the baseline for: who (actors on the lane's own entries), in_progress, the farm delta (new/moved yaks, new notes, needs). Fall back to today's merge-base when the reflog is missing or its first commit is unknown, and say `(vs viewer)` so nothing is silently wrong. Keep ahead/behind as they are (they are about the viewer).

## Evidence
Tests with temp repos: a worker forked from a coordinator that already has its own notes shows only the worker's entries; two lanes forked from the same coordinator do not share labels; a landed (squashed) lane shows its own commit, not the history before it; no reflog falls back and says so. A real run in ~/src/yaks must show one distinct actor set per worker lane, and `inp-1` in inp-1's lane only.

---
▸ 2026-10-06T21:32:29Z [delta-lead]
Terminology decision (Joel, 2026-10-06, via the naming subthread): a "lane" is now a SHED, everywhere, no aliases; `yaks lanes` becomes `yaks sheds`. Text above this note, and all history, keeps the word "lane": read it as "shed". Not decided: "clip" (what a shed has produced) and "barn" (the primary checkout). The rename itself is yaks-dfca; it lands before the baseline fix (yaks-37fa) because both edit the same file.

---
▸ 2026-10-06T21:36:27Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T21:36:27Z [delta-lead]
Claim (coordinator delta-lead), after the rename landed (yaks-dfca, 7f49fe2). Read "lane" in the description as "shed"; the file is now `src/sheds.rs`, the command `yaks sheds`, the types `Shed`, `ShedKind`, `ShedFarm`, the Farm method `Farm::sheds`. Owner: worker bas-1.

## Hints for the fix
- The fork point of a shed is the OLDEST entry of its HEAD reflog: the first line of `<git-dir>/logs/HEAD` (`git -C <shed> rev-parse --git-dir`), whose second field is the commit the shed started from. For a Delta clone that is the coordinator's HEAD at spawn time; for a `git worktree add -b` shed it is the branch creation. I checked it on the real checkouts: each worker shed's first reflog entry is its spawn base, and commits since it are exactly the worker's own (inp-1's shed: 1 commit since its fork against 3x attributed by the viewer's merge-base).
- The reflog can be missing, truncated or expired, and its first commit may be unknown to the viewer's object store; then fall back to today's merge-base and say so in the output (a short `(vs viewer)`-style marker, both in the table and in JSON), exactly as the current fallback does.
- The base's committed `.yaks` is already read from the viewer's repo (ls-tree plus one cat-file) into a temp dir that is removed: reuse that, only the base commit changes. Delta clones share objects with the viewer (alternates) and git worktrees share the repo, so the fork commit is normally present.
- `ahead`/`behind` stay as they are: they describe the shed relative to the viewer, not the shed's own work.
- A shed that has just been spawned (no commits since its fork) must show no own changes and no label.

## Judge
The coordinator re-runs the gate and runs `yaks sheds` from ~/src/yaks, where each worker shed must show only its own actors and yaks (inp-1's shed: inp-1 only; no `delta-lead` inherited from the thread that spawned it). Forbidden: behaviour changes other than the baseline, any other yak, coordination skills' prose (the coordinator updates it).

---
▸ 2026-10-06T21:37:02Z [bas-1]
Start Tue Oct  6 21:37:02 UTC 2026; yaks path: /Users/joel/src/yaks/.delta/worktrees/f2hdbpew7kgx/yaks/.yaks/shaving/yaks-37fa.md; checkout /Users/joel/src/yaks/.delta/worktrees/f2hdbpew7kgx/yaks. Before-binary copied to /tmp/bas1/yaks-before (outside repo, scratch only).

---
▸ 2026-10-06T21:41:08Z [bas-1]
Decision: the fork point is validated before use: the reflog's first commit must be an ancestor of the shed's HEAD (`git merge-base --is-ancestor <fork> <HEAD>` in the viewer's repo), else fall back to the merge-base. Rejected: trusting the first line blindly. A shed rebased onto another root (or whose first line names a commit this repo lacks) would otherwise be measured from a commit that is not in its history, and "since the fork" would be meaningless.

---
▸ 2026-10-06T21:41:08Z [bas-1]
Decision: three baselines, one marker each. Fork point: no marker, JSON `farm.vs: "fork"`. Merge-base fallback: ` (vs merge-base)` on the farm line AND on the label line (the label is what showed the inherited actors), JSON `"merge-base"`. Neither readable: unchanged ` (vs this checkout)` / `"checkout"`. The brief says "`(vs viewer)`-style"; I wrote `vs merge-base` because "viewer" is not a term this CLI or its docs use ("this checkout" is). Rejected: one shared `(vs viewer)` for both fallbacks (hides which one was used); marker on the farm line only (the label line would look authoritative). JSON `farm.base` is still the short sha of whichever base was used.

---
▸ 2026-10-06T21:41:08Z [bas-1]
Decision: if the fork commit is valid but its farm cannot be read (no `.yaks` committed there), try the merge-base next before falling back to the checkout working tree, instead of jumping straight to the checkout. Same chain as before for the merge-base; the marker tells which base was used. Rejected: fork-or-checkout only, which would turn a shed that created its farm itself into a noisy "vs this checkout" diff when a merge-base farm exists.

---
▸ 2026-10-06T21:41:08Z [bas-1]
Decision: `Shed.farm_base` is now `Option<FarmBase { kind: BaseKind::{Fork, MergeBase}, sha }>` rather than a second field next to `Option<String>`: the kind and the sha cannot disagree, and `None` still means "compared with this checkout". Public struct only used inside src/sheds.rs (and its tests); no TUI/other consumer.

---
▸ 2026-10-06T21:41:08Z [bas-1]
Decision: left alone, as scoped: `ahead`/`behind` (still vs the viewer), the coordination skills' prose (the coordinator updates it), TUI. The `(vs this checkout)` wording is unchanged. Observed, not changed: the coordinator's own shed (xp49...) still lists 16 actors in the real run, because its thread forked long ago and everything since really is in its own history; that is the correct reading of "since this shed forked".

![before](artifacts/yaks-37fa/before.txt)

![sheds-after](artifacts/yaks-37fa/sheds-after.txt)

---
▸ 2026-10-06T21:41:31Z [bas-1]
Evidence: real `yaks sheds` from /Users/joel/src/yaks, before (/tmp/bas1/yaks-before = coordinator binary) and after (this change), attached in full as before.txt / sheds-after.txt. Worker sheds, who/farm lines (path dir, HEAD):

| shed | before | after |
|---|---|---|
| 8jspjv836gbr 5e5a1e9 | who: delta-lead, inp-1 · farm: 1 new, 1 moved, 12 new notes | who: inp-1 · farm: 1 moved, 10 new notes |
| ccae3bj40fr0 b750092 | who: delta-lead, ntf-1 · farm: 1 new, 1 moved, 2 new notes | who: ntf-1 · farm: 1 moved, 4 new notes |
| 486hnt20begb bf0a090 | who: delta-lead, inp-1, ntf-1, Joel Webber, agt-1 · 48 new notes | who: agt-1 · 1 moved, 11 new notes |
| 3kkcfcww892t db7cc43 | who: delta-lead, inp-1, ntf-1, Joel Webber, agt-1, brf-2 · 52 new notes | who: brf-2 · shaving: yaks-2c22 · 1 new notes |
| vk8z2vjzgdqh d90e8a1 | who: 7 actors | who: shd-1 · 1 moved, 4 new notes |
| 0725169 shed (xw51kfjdpqy5) | who: delta-lead, inp-1, ntf-1, Joel Webber, brf-1 | who: brf-1 · shaving: yaks-2c22 |

No `(vs ...)` marker appears: every real shed has a usable reflog. Tests added in src/sheds.rs (all pass): a_worker_forked_from_a_coordinator_with_notes_shows_only_the_workers_entries, a_freshly_spawned_worker_has_no_changes_and_no_label, two_sheds_forked_from_the_same_coordinator_share_no_labels, a_landed_squashed_shed_shows_its_own_commit_not_the_history_before_it, a_missing_reflog_falls_back_to_the_merge_base_and_says_so, an_unusable_reflog_falls_back_to_the_merge_base (unknown commit / not a sha / empty / garbage), a_fork_point_that_is_not_an_ancestor_of_the_shed_falls_back; a_shed_that_is_only_behind... now asserts JSON vs "fork" and no marker.

---
▸ 2026-10-06T21:41:46Z [bas-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-06T21:41:51Z [bas-1]
Summary: `yaks sheds` now measures a shed from its own fork point (oldest HEAD reflog entry, validated as an ancestor of the shed's HEAD), falling back to the merge-base with this checkout and marking it `(vs merge-base)` on the farm and label lines (JSON `farm.vs`: fork | merge-base | checkout). Changed: src/sheds.rs (+7 tests), `Sheds` help in src/main.rs, the `sheds` rows in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md. Gate `cargo test -p yaks`: PASS. Real run: each worker shed shows only its own actors (table above).

---
▸ 2026-10-06T21:41:51Z [bas-1]
moved: shaving -> shorn
