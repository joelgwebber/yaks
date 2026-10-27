---
id: yaks-07fe
title: 'yaks sheds is slow (~1.2 s for 4 sheds): profile it and make the common path cheap'
status: shorn
type: task
priority: 2
created: '2026-10-12T16:04:57Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-38dc
labels:
- cli
- delta
needs: agent
---

`yaks sheds` takes ~1.2 s on this machine (4 live sheds plus 51 orphaned Delta pins, release build, measured 2026-10-12); `yaks inbox --sheds` about the same. Joel: worth finding out why before the TUI (yaks-38dc, approach B) builds on it with a background refresh.
Profile first, then fix. Suspects, not measured: one `git` process per question per shed (rev-parse, symbolic-ref, merge-base, two rev-list counts, status --porcelain, reflog read, ls-tree + cat-file for the base farm); resolving every pin group in the host, including the 51 gone ones, on every call; reading each shed's whole farm plus a base farm materialized into a temp dir; the reference farm read once more for `landed`.
Goal: know where the time goes (per-shed and per-step timings), then make the common path cheap: e.g. skip or cache gone pins, batch git calls, skip the base-farm read when a shed's farm and HEAD are unchanged since the last call (a derived, per-user cache only: the AGENTS.md rule for any index), and separate what can be event-driven (farm mtime) from what needs git.
Evidence: before/after timings on this machine and on ~/src/j15r.

---
▸ 2026-10-13T00:11:39Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-13T00:11:39Z [delta-lead]

Claim (delta-lead, yakherd): owner perf-1 (Sonnet worker, its own Delta thread). Runs in parallel with yaks-8814 (needs-1) and yaks-98f1 (the TUI Sheds tab, delta-lead).

Goal: know where `yaks sheds` spends its time, then make the common path cheap without changing any output.

Measured by delta-lead, 2026-10-12, release build in the yakherd's clone (10 sheds incl. 6 old-layout, 53 orphaned pins): `yaks sheds` 2.0-2.5 s wall, 0.5 s user + 0.6-1.0 s sys; a git shim counted 43 git subprocesses (13 rev-parse, 4 rev-list, 4 merge-base, 4 config, 2 status, 2 cat-file, 2 ls-tree, ...) summing to about 0.95 s, and git ran only for the current checkout and the primary. The rest of the wall time is unexplained: find it.

Scope: src/sheds.rs (discover, inspect, host, pinned, orphaned_pins, base_farm, reference_tasks, load_farm, the git helpers), and src/main.rs `Command::Sheds` only if a combined call needs it. Keep the signatures of `sheds::discover`, `sheds::changes`, `sheds::orphaned_pins`, `sheds::farm_line`, `sheds::is_outstanding` and `sheds::label_line` (the TUI's Sheds tab calls them). Adding a function that returns the sheds and the orphaned pins from one pass is fine.
Out of scope: `outstanding_ids` and the landed rules (yaks-8814 owns them), anything in src/tui/, any output change. A persistent cache (even a per-user, derived one, per AGENTS.md) is a design choice: `yaks ask` before adding one.

Evidence: per-step timings before/after (how you measured them, in a note); `yaks sheds` and `yaks sheds --json` output byte-identical before/after on this machine (diff them, `farm_activity`/`FARM ACTIVE` timestamps aside if they move); `yaks changes main` and `yaks inbox --sheds` likewise; full `cargo test --release` green. Judge: delta-lead at landing, re-timed on the real machine.

---
▸ 2026-10-13T00:12:57Z [perf-1]

Start: Tue Oct 13 00:12:57 UTC 2026
yaks path yaks-07fe: /Users/joel/src/yaks/.delta/worktrees/53smxwrxenfm/yaks/.yaks/yaks-07fe.md

---
▸ 2026-10-13T00:26:33Z [perf-1]

Profile (before binary, release, this machine: 4 live sheds incl. current, 61 orphaned pins, 14 cores). Measured with a TEMPORARY Instant wrapper around `git_raw`, `newest_mtime`, `load_farm` and `discover` (removed before committing; a python git shim was rejected: it cost ~0.3 s per call and drowned the signal).
- `yaks sheds` wall 1.2-1.5 s; user 0.28, sys 0.61.
- `discover` after the toplevel probe: 1.10-1.24 s. checkout_paths 100-117 ms of that.
- 80 git spawns per `yaks sheds`, each 9-15 ms wall (`git rev-parse HEAD` alone is ~10 ms here), all SERIAL: 1.0 s. That is the whole story. They are mostly redundant: `rev-parse --absolute-git-dir` x4 per shed (title, fork_point, host x2), `rev-parse HEAD` + `--short HEAD` separately, two `rev-list --count` for one symmetric difference, `merge-base --is-ancestor X X` when the fork point IS the head, `rev-parse --short` + `ls-tree` + `cat-file` repeated for each shed with the SAME base (3 of 4 sheds here fork from 280767f), `worktree list` x3 for one repo, `host()` + `for-each-ref` run twice (discover, then `orphaned_pins`), the 3 `config core.worktree` calls serial, toplevel/git-dir/common-dir/shallow as 4 calls.
- Filesystem is NOT the cost: `newest_mtime` 1.8-2.4 ms per farm, `load_farm` 9-46 ms per farm (first read cold), the 61 gone pins are only an `is_dir` each (no process).
- Per-step git totals, one run: rev-parse 32 calls, config 11, symbolic-ref 4, merge-base 8, rev-list 8, status 4, ls-tree 4, worktree list 3, for-each-ref 2 (+ 4 cat-file spawned outside `git_raw`).
Plan: (1) run the per-shed inspection and the pin resolution on a bounded thread pool, order preserved; (2) cut git calls: fold rev-parse probes into one, one rev-list --left-right, skip the trivially-true is-ancestor, one worktree list per repo, one for-each-ref via a combined `discover_all`; (3) memoize the base farm per (base commit, farm path) within one call and parse the blobs in memory instead of a temp dir.

---
▸ 2026-10-13T00:49:16Z [perf-1]

Decision: no persistent cache. The wall time was 80 serial `git` spawns at ~10 ms each, not filesystem work, so reducing and overlapping the spawns got 4x without a derived-cache design (which is yours to approve). Rejected: a per-user cache keyed on (HEAD, farm mtime), a `libgit2`/`gix` dependency (forbidden), a persistent `git cat-file --batch-check` daemon.
Decision: concurrency by `std::thread::scope` (no new dependency): `par_map` (8 workers, order-preserving) over the sheds and over the pin groups; scoped threads for the independent probes inside one shed and inside discovery. Rejected: rayon (new dep), tokio.
Decision: fewer spawns, same answers. `rev-parse --absolute-git-dir HEAD --short HEAD` as one call; `--absolute-git-dir --git-common-dir` as one call in `host`; `rev-list --left-right --count HEAD...<shed>` replaces two `--count` calls; when the shed's HEAD equals ours, merge-base is HEAD and ahead/behind are 0/0 (no spawn); a fork point equal to the shed's HEAD or to its merge-base skips `merge-base --is-ancestor` (an ancestor by definition; the call still runs otherwise); `worktree list` in the host is run once (its first entry is the host checkout, the list is the same from there) instead of twice.
Decision: base farms memoized per `discover` by (base commit, farm path in checkout) in a `OnceLock` cell, so the sheds forked from one commit read that farm once. Rejected: parsing blobs in memory instead of the temp dir (touches `load_farm`/parse, no measurable gain: 9 ms per load).
Decision: new `sheds::discover_all(cwd, farm_root) -> (sheds, orphaned pins)` from one pass (the host and pin groups were read twice: discover, then `orphaned_pins`); `Command::Sheds` uses it. `discover` is now `discover_all(..).0`. `orphaned_pins` keeps its signature (the TUI tab calls it) and is marked `#[allow(dead_code)]` in the CLI binary because only the TUI tab will use it. `changes`, `farm_line`, `is_outstanding`, `label_line` untouched. `outstanding_ids`/`landed` untouched.

---
▸ 2026-10-13T00:49:16Z [perf-1]

Timings, wall clock, 7 runs each (min / median / max, seconds), this machine (4 sheds incl. current, 61 orphaned pins), measured by a python subprocess loop with the before binary (target/yaks-before, built before any edit) and the after binary back to back:
- `yaks sheds`: before 1.24 / 1.28 / 1.69, after 0.31 / 0.32 / 0.40
- `yaks sheds --json`: before 1.14 / 1.16 / 1.21, after 0.31 / 0.31 / 0.34
- `yaks changes main`: before 1.25 / 1.26 / 1.37, after 0.41 / 0.44 / 0.48
- `yaks inbox --sheds`: before 1.16 / 1.20 / 1.23, after 0.35 / 0.36 / 0.41
- ~/src/j15r (1 checkout; `sheds`): before 0.29 / 0.30 / 0.37, after 0.13 / 0.13 / 0.14; `sheds --json` 0.25/0.26/0.29 -> 0.12/0.13/0.14; `inbox --sheds` 0.24/0.25/0.28 -> 0.13/0.13/0.14.
Spawns: 80 git subprocesses before; after, a temporary timeline (Instant stamps, removed) showed the gather phase done at 112 ms (was ~330 serial), per-shed git at ~150-240 ms with 4 sheds overlapping, the longest chain being rev-parse -> merge-base -> rev-list -> is-ancestor -> ls-tree.
Parity: `yaks sheds`, `sheds --json`, `changes main`, `inbox --sheds` byte-identical (cmp) between before and after, run before/after/before back to back, and the two before runs equal each other (no FARM ACTIVE movement). Also `yaks sheds` in ~/src/j15r identical.
Left on the table (not done): `changes main` is still ~0.1 s above `sheds` because `shed_yak_changes` re-reads the base farm and reference farm serially after discover (a combined call would need src/main.rs changes beyond this yak); the remaining `sheds` wall is ~14 spawns on the critical path at ~10 ms and ~0.5 s sys, mostly fork/exec cost of git on macOS.

---
▸ 2026-10-13T00:50:05Z [perf-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T00:50:12Z [perf-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-07fe` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-07fe --drop-request`, which removes this request note, so no new note is left behind.
