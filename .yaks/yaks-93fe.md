---
id: yaks-93fe
title: 'sheds: a plain git worktree of a Delta clone is taken for the primary (named main, measured vs origin/main)'
status: shaving
type: bug
priority: 3
created: '2026-10-13T00:25:06Z'
updated: '2026-10-13T01:29:44Z'
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
