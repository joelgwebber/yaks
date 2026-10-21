---
id: yaks-93fe
title: 'sheds: a plain git worktree of a Delta clone is taken for the primary (named main, measured vs origin/main)'
status: shorn
type: bug
priority: 3
created: '2026-10-13T00:25:06Z'
updated: '2026-10-13T01:55:27Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Seen 2026-10-13 by delta-lead while checking the Sheds tab (yaks-98f1) on the real machine: `git worktree add --detach <scratch> HEAD` inside this thread's Delta clone (`.delta/worktrees/gcmb7srbmkvv/yaks`) made a shed that `yaks sheds` lists as

```
  worktree  (detached)  503fca3  +0 -0  1  ...  <scratch>/tabcheck
  name: main
  who: delta-lead, Joel Webber, needs-1, tabcheck · shaving: ... · active (vs origin/main)
  farm: 1 new, 3 moved, 19 new notes, 1 needs (vs origin/main)
```

so two rows are named `main`, and the worktree is measured from origin/main (crediting it with everything the clone did) instead of from its own fork point. The real primary (`/Users/joel/src/yaks`) is listed correctly too. Probably `checkout_paths` / `Found::primary` marks a worktree found through the CURRENT clone's `git worktree list` as primary. Expected: primary only for the host checkout; a worktree of a clone is a plain shed (named by its actor or branch, fork-point baseline). Rare in practice (workers do not make worktrees inside Delta clones), so P3. Fix after yaks-07fe lands, since it restructures discovery.

---
▸ 2026-10-13T01:29:43Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-13T01:29:44Z [delta-lead]

Claim (delta-lead, yakherd): owner disco-1 (Sonnet worker). Parallel with badge-1 and inbox-1 (TUI only, no overlap with src/sheds.rs).

Goal: a plain `git worktree` added inside a checkout that is itself a Delta clone (or any non-primary checkout) is a plain shed: not `primary`, so not named `main`, measured from its own fork point, and the real primary (the host checkout) stays the only `primary`. Find where `Found::primary` / `Shed::primary` is decided (`checkout_paths`, `gather`, `worktree_entries`, `host_checkout` in src/sheds.rs) and fix the rule, not the example.

Scope: src/sheds.rs (discovery: the functions above and their tests), tests/ if an integration test fits better (a fixture: a host repo, a Delta-like clone with refs/delta pins as the existing tests build them, and a `git worktree add` inside the clone). Docs only if the `sheds` row's wording about the primary changes (docs/cli.md, the yaks skill's `sheds` row, `Sheds` clap help in src/main.rs).
Out of scope: src/tui/ (two TUI workers), `outstanding_ids`/landed, performance.

Evidence (fail first): a test with exactly that layout: the worktree is listed, not primary, named by its actor/branch/dir (not `main`), baseline `fork` (or `merge-base`), and exactly one shed is primary; existing discovery tests unchanged. Real machine: from this repo's own clone, `git worktree add --detach <your scratch dir>/wt HEAD`, then `yaks sheds` shows it as a plain shed and `main` once (paste both before/after); remove the worktree after. Output for every other shed byte-identical before/after (`yaks sheds`, `--json`). Full `cargo test --release`. Judge: delta-lead at landing.

---
▸ 2026-10-13T01:31:25Z [disco-1]

disco-1 start: 2026-10-13T01:31:21Z UTC (date -u).
yaks path yaks-93fe: /Users/joel/src/yaks/.delta/worktrees/2fdsqr81jxvy/yaks/.yaks/yaks-93fe.md
Before binary kept at target/yaks-before.

---
▸ 2026-10-13T01:33:15Z [disco-1]

Test written first: `sheds::tests::a_git_worktree_of_a_delta_clone_is_a_plain_shed_not_the_primary` (host checkout t, Delta clone c1 with its own commit + new yak, `git worktree add --detach wt` inside c1; asserts from c1, t and wt: paths [t, c1, wt], exactly one primary == t, wt not primary, name "wt" not "main", baseline Fork/MergeBase).
Failure before the fix (`cargo test --release a_git_worktree_of_a_delta_clone`):
  assertion `left == right` failed: [t, t/.delta/worktrees/d1/repo, wt]
    left: 2
   right: 1
i.e. two sheds are primary (the host checkout AND the worktree).
Cause: `gather` marks `primary |= i == 0` for the first CHECKOUT-ish entry of every `git worktree list` it reads; in a Delta clone entry 0 is the clone's separate git dir (no `.git`, dropped by `worktree_entries`'s verdict), so entry 1, the plain worktree, got index 0 after the filter and was marked primary.

---
▸ 2026-10-13T01:37:06Z [disco-1]

Decision: the rule for "primary" is "entry 0 of a `git worktree list`, and only when entry 0 is itself a checkout" (git always lists the main working tree first). `gather` now passes the raw entries (path + verdict) to `add_worktrees` and uses the entry's index in the UNFILTERED list. In a Delta clone entry 0 is the clone's separate git dir (no `.git`, verdict Err), so the list has no primary and a plain worktree there stays a plain shed; the host checkout stays the primary via the host's own list. Rejected: (a) "primary only for the host checkout" (breaks a plain repo with no Delta, where the repo's main working tree is primary and `host_checkout` is the same thing but via a different path; more code, two rules); (b) "primary = the entry whose `.git` is a directory" (a separate-git-dir or bare-ish checkout would lose it; guesses from layout instead of what git says).
Decision: removed `worktree_paths`/`entries_paths` (now unused) rather than keeping dead helpers.
Decision: no docs/clap/skill wording changed: `main` = primary checkout stays true; only a doc comment on `Found::primary` (src/sheds.rs) was sharpened. Grep of docs/cli.md, SKILL.md, main.rs for "primary" found no claim the fix contradicts.
Known limit (not this yak, same before/after): the host's own `git worktree list` cannot see a worktree registered in a clone's git dir, so from the host checkout (and from inside the worktree) the worktree/clone is not listed; the test checks views from the clone and from the worktree itself.
Real machine (my own clone, `git worktree add --detach $DELTA_SCRATCH_DIR/wt HEAD`), `yaks sheds`:
BEFORE (target/yaks-before):
  worktree  (detached)  5392019  +0 -0  0  2026-10-13T01:36:45Z  <scratch>/wt
  name: main
  who: delta-lead · shaving: yaks-1ea3, yaks-7204, yaks-93fe, yaks-ed6d · active (vs origin/main)
  farm: 2 new, 2 moved, 7 new notes (vs origin/main)
AFTER:
  worktree  (detached)  5392019  +0 -0  0  2026-10-13T01:36:45Z  <scratch>/wt
  name: wt
  farm: no changes of its own
`name: main` appears once in the AFTER output (the real primary /Users/joel/src/yaks, `(vs origin/main)`), twice BEFORE. Worktree removed afterwards (`git worktree list` shows only the clone).
With no extra worktree, back to back: `yaks sheds` and `yaks sheds --json` byte-identical before/after (cmp; two runs, no FARM ACTIVE time moved between them), `yaks discover` still runs.

---
▸ 2026-10-13T01:39:39Z [disco-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T01:53:35Z [disco-1]

Flake seen once, unrelated: in one full `cargo test --release` run `tests/sheds_layouts.rs::a_flat_shed_forked_before_the_migration_falls_back_to_the_next_base_not_to_nothing` failed (`vs` was "checkout", expected "merge-base"). It passes on the stashed baseline (my src/sheds.rs change stashed), and in 3 consecutive full runs and 3 consecutive runs of that suite after, plus the verify run. That layout is a plain repo + `git worktree add -b` shed, where the unfiltered entry index equals the old filtered one, so the change cannot alter it; I could not reproduce it. Likely a timing flake (same-second commits / reflog) in that test; reporting, not fixing (out of scope).
Summary: `gather` (src/sheds.rs) now decides `primary` from the raw `git worktree list` index (entry 0, only if it is a checkout), so a plain worktree inside a Delta clone is a plain shed (named by dir/branch, fork baseline), the host checkout stays the only primary. New test `a_git_worktree_of_a_delta_clone_is_a_plain_shed_not_the_primary`; removed dead `worktree_paths`/`entries_paths`; sharpened `Found::primary` doc comment.

---
▸ 2026-10-13T01:55:23Z [disco-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T01:55:27Z [disco-1]

moved: shaving -> shorn
