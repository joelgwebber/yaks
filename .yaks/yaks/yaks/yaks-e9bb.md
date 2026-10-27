---
id: yaks-e9bb
title: 'Sheds: one baseline per shed, its fork point, from every viewer (drop the viewer-relative sync rule)'
status: shorn
type: task
priority: 2
created: '2026-10-10T18:43:28Z'
updated: '2026-10-27T01:18:53Z'
parent: yaks-ee0a
labels:
- cli
- delta
verify: cargo test --release
---

Implements decisions 1-5 of the shed semantics (see the Decisions note on yaks-ee0a). Today a shed's baseline is the NEARER of its fork point and its merge-base with the VIEWER (`nearer_baseline`, yaks-22f1), so two viewers can disagree about one shed, and the sync case needs the viewer's merge-base, which a shallow clone cannot always give.

Change: the baseline is the shed's fork point (oldest reflog entry, validated as an ancestor of its HEAD) from every viewer; the shed side is its working tree. Remove `nearer_baseline`, `BaseKind::Sync` and the `sync` value of `farm.vs` (JSON; record the change as a Decision), keep the merge-base fallback for a missing/unusable reflog (`vs merge-base`, still viewer-relative: say so). The primary's own row, once listed (yaks-ad6d), is measured from `origin/main`.

Spike results (2026-10-10, real data and a scratch repo), so nobody re-derives them:
- `git worktree add -b`, `--detach` and an existing branch all write the creation as the FIRST reflog entry (`0000.. <start commit>`), so the fork point is the start commit, as for a Delta clone (its first entry is the clone's HEAD at creation).
- A later `git merge main` in the shed is an ordinary reflog entry (`merge main: ...`); the first entry never moves. So after a sync the fork-only baseline still credits the shed with what it merged in. On this thread that is three yaks of Joel's and the coordinator's (yaks-3479, a4d8, cf2e), shown with their own actors: the accepted redundancy.
- Shallow: Delta's thread sync left the primary (~/src/yaks) and this clone shallow, boundary 2c28d23, a pinned commit (also yaks-1b39, yaks-a398). ~/src/j15r and its clones are not shallow. Fork-only needs no merge-base with the viewer, so it is less sensitive to a shallow viewer; the `is-ancestor` check of the fork point within the shed's own history remains.

Evidence: tests for two viewers (primary and a sibling) giving the same answer for one shed that merged the viewer's main after forking; a worktree shed; a clone; docs parity (docs/cli.md, README, yaks skill, `Sheds` help). Supersedes the sync part of yaks-22f1.

---
▸ 2026-10-10T18:56:18Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-10T18:56:19Z [delta-lead]

Claim (coordinator). Owner: base-1. Spec = the description above and the 2026-10-10 Decisions on yaks-ee0a (read them: decisions 1-3). Joel confirmed that dropping the `sync` value of `farm.vs` from the JSON is fine.

Scope by symbol (src/sheds.rs): `nearer_baseline` (delete), `BaseKind::Sync` and its `as_str`, the `fork` candidate in `GitState` and where it is built (`fork_point(...).map(nearer_baseline ...)`), the baseline list in `inspect` (the `bases` vector), the `(vs merge-base)` text, the module docs on baselines (lines ~10-23), and every test that names `Sync`/`sync`. Docs: the `vs` / baseline wording in docs/cli.md, README.md, `.agents/skills/yaks/SKILL.md`, the `Sheds` help in src/main.rs.

Keep: the fork point validation (ancestor of the shed's HEAD), the merge-base FALLBACK when there is no usable reflog (`vs merge-base`, still viewer-relative: say so in the doc), `vs this checkout` when no base farm can be read, ahead/behind exactly as they are (commits are a different question).

Out of scope: `yak_changes` and its counts and renders (another worker owns them: do not touch), landed/stale, the current-shed row, the primary's `origin/main` baseline (yaks-ad6d). Put your tests beside the existing baseline tests; the other worker adds `yak_changes` tests elsewhere.

Evidence (tests FAIL first): one shed forked from commit F that later merged the VIEWER's `main`: seen from two different viewers (the primary and a sibling shed) it reports the SAME baseline (F), the same changes and the same label; before the change they differ. A git-worktree shed and a Delta-clone fixture both use their first reflog entry. The old `sync` test is rewritten to expect the fork baseline (and the extra changes the merge brought, with their actors), not deleted. `farm.vs` never says `sync`. Real machine: run `yaks sheds --json` from ~/src/yaks before and after and attach both; the differences must be exactly the cases the sync rule used to cover. Record a `Decision:` for the removed JSON value. Judge: coordinator.

---
▸ 2026-10-10T18:57:48Z [base-1]

Start 2026-10-10T18:57:48Z (base-1). yaks path: /Users/joel/src/yaks/.delta/worktrees/qkpp5wdvfm8p/yaks/.yaks/shaving/yaks-e9bb.md. Plan: tests first (fail against the nearer_baseline rule), then drop nearer_baseline/BaseKind::Sync/farm.vs sync, update docs+help.

---
▸ 2026-10-10T18:58:18Z [base-1]

Failing-first output (before any code change), 'cargo test --release --bin yaks sheds::tests::a_': 4 failed, 17 passed.

The three viewer tests fail with 'left: Some(FarmBase { kind: Sync, sha: ... }) right: Some(FarmBase { kind: Fork, sha: ... })' (the viewer-relative rule picks the viewer's merge-base); the clone test fails with 'left: None right: Some(FarmBase{kind: Fork..})' which I am investigating (likely fixture, not the rule).

```text
failures:
---- sheds::tests::a_delta_clone_is_measured_from_its_first_reflog_entry_from_every_viewer stdout ----
thread 'sheds::tests::a_delta_clone_is_measured_from_its_first_reflog_entry_from_every_viewer' (31581783) panicked at src/sheds.rs:2914:13:
assertion `left == right` failed: viewer /private/var/folders/j3/ngc8z3h567v_l779rt0gtdlw0000gp/T/delta-terminal-.delta-fs-DSyMl2/yaks-sheds-fork-clone-83415-0/repo
  left: None
 right: Some(FarmBase { kind: Fork, sha: "e921959" })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
---- sheds::tests::a_shed_that_merged_the_viewer_still_shows_everything_since_its_fork stdout ----
thread 'sheds::tests::a_shed_that_merged_the_viewer_still_shows_everything_since_its_fork' (31587205) panicked at src/sheds.rs:2769:9:
assertion `left == right` failed
  left: Some(FarmBase { kind: Sync, sha: "52442b7" })
 right: Some(FarmBase { kind: Fork, sha: "d75e4f2" })
---- sheds::tests::a_shed_the_viewer_merged_still_shows_everything_since_its_fork stdout ----
thread 'sheds::tests::a_shed_the_viewer_merged_still_shows_everything_since_its_fork' (31587559) panicked at src/sheds.rs:2769:9:
assertion `left == right` failed
  left: Some(FarmBase { kind: Sync, sha: "31e779d" })
 right: Some(FarmBase { kind: Fork, sha: "d75e4f2" })
---- sheds::tests::a_shed_that_merged_the_viewers_main_reads_the_same_from_every_viewer stdout ----
thread 'sheds::tests::a_shed_that_merged_the_viewers_main_reads_the_same_from_every_viewer' (31587305) panicked at src/sheds.rs:2769:9:
assertion `left == right` failed
  left: Some(FarmBase { kind: Sync, sha: "08ef270" })
 right: Some(FarmBase { kind: Fork, sha: "d75e4f2" })
```

[yaks-e9bb-sheds-old](yaks-e9bb/yaks-e9bb-sheds-old.json)

[yaks-e9bb-sheds-new](yaks-e9bb/yaks-e9bb-sheds-new.json)

---
▸ 2026-10-10T19:01:19Z [base-1]

Real-machine comparison (old = PATH yaks 0.0.14, new = ./target/release/yaks, both `sheds --json` run from this clone, read-only; attached as yaks-e9bb-sheds-old.json / -new.json). Same 4 rows; 2 identical, 2 differ, and both differences are exactly the old `sync` cases:
- `main` (primary, ~/src/yaks, listed from this Delta clone): old `vs: sync, base 4d37dce`, 0 changes (the clone had merged the primary's main, so the nearer merge-base was the primary's tip). New `vs: fork, base f5dce34` = the primary's own first reflog entry (an old commit from Sep, its reflog is 187 lines long): 154 added, 23 moved, 172 notes, 3 needs, 61 actors. That is the accepted cost of "fork point always", and it is a nonsense number for the PRIMARY row: the primary has no fork point. The coordinator spec says the primary's baseline is origin/main and is yaks-ad6d's job (out of scope). See Decision below.
- `yak-shedding` (a Delta clone, this thread's coordinator-ish shed): old `sync, base ece5bf8`, 2 notes; new `fork, base 9e4ca2a` (v0.0.14, where the clone was made): 8 added, 3 moved, 9 notes, who +1 (Joel Webber), in_progress gains yaks-e9bb: the yaks the shed merged in from main, shown with their actors (the accepted redundancy).
- `edit-1...` and `edit-2...`: identical (both were already `fork`).

Nothing else differs (ahead/behind, dirty, activity unchanged). `farm.vs` is never `sync` in the new output.

---
▸ 2026-10-10T19:01:19Z [base-1]

Decision: baseline is the shed's fork point from every viewer; `nearer_baseline` and `BaseKind::Sync` deleted. Rejected: (a) keep `nearer_baseline` but compute it against a fixed reference such as origin/main (would still make a shed's baseline move when main moves, and needs a reference every checkout may lack); (b) keep Sync only when the SHED merged the viewer (still viewer-dependent, as the two-viewers test shows); (c) skip merged-in commits by reading merge parents (heuristic, shallow-fragile, and decision 2 of yaks-ee0a explicitly accepts repeats).

Decision: the JSON value `farm.vs == "sync"` is removed (Joel agreed). Values are now `fork`, `merge-base`, `checkout`. Rejected: keep `sync` in the enum for compatibility (nothing could ever produce it), or alias it to `fork`. Docs/help/README/skill say so.

Decision: the merge-base FALLBACK (`vs merge-base`) stays viewer-relative, and the docs say so in those words, as the spec asks. Rejected: dropping the fallback (a shed with an expired reflog would then fall to `vs this checkout`, a weaker answer) or inventing a viewer-independent fallback (nothing in git gives one).

Decision: the PRIMARY row, when listed from a clone, is measured like any other shed (fork point = first entry of its own HEAD reflog), not special-cased to origin/main, because the spec puts the primary's origin/main baseline in yaks-ad6d. Consequence, visible in the real-machine diff: `main` seen from this clone goes from `0 changes (sync)` to a huge number from an old reflog start. Rejected: special-casing it here (scope creep that would overlap yaks-ad6d's design) . COORDINATOR: yaks-ad6d should land before or with this for the primary row to read sensibly, or accept it as a temporary regression.

Decision: no change to `ahead`/`behind` or to `GitState.merge_base` (still needed for the fallback and the counts), as the spec says.

Decision: the Delta-clone test fixture pins the clone's HEAD in the host by fetching `refs/delta/d1/repo/<sha>`, as Delta does for every commit, because without it the clone's commits are unknown to the viewer's object store and `fork_point`'s `is-ancestor` check legitimately returns None (the first run of that test failed for this reason, `left: None`; that is a fixture gap, not a bug in the rule).

Tests (beside the baseline tests, src/sheds.rs): rewrote the old sync pair (`a_shed_the_viewer_merged_still_shows_everything_since_its_fork`, `a_shed_that_merged_the_viewer_still_shows_everything_since_its_fork`) to expect the fork baseline, the merge's yak notes and their actors; new `a_shed_that_merged_the_viewers_main_reads_the_same_from_every_viewer` (primary and sibling viewers: same baseline, same FarmDelta, same JSON farm object, same label) and `a_delta_clone_is_measured_from_its_first_reflog_entry_from_every_viewer`; `a_shed_keeps_its_fork_point_even_when_the_viewer_moved_on` renamed. The squash test comment updated.

---
▸ 2026-10-10T19:02:17Z [base-1]

verify: `cargo test --release` -> PASS (exit 0)

---
▸ 2026-10-10T19:02:23Z [base-1]

Summary: baseline is the fork point from every viewer (nearer_baseline, BaseKind::Sync, farm.vs 'sync' removed); merge-base fallback kept and documented as viewer-relative; docs/cli.md, README, yaks skill, Sheds/Changes help updated; 4 new/rewritten tests fail-first then pass; full cargo test --release PASS. Open for the coordinator: the primary row from a clone now reads from its own old reflog start until yaks-ad6d (origin/main baseline) lands.

---
▸ 2026-10-10T19:02:23Z [base-1]

moved: shaving -> shorn
