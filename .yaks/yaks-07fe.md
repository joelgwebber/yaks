---
id: yaks-07fe
title: 'yaks sheds is slow (~1.2 s for 4 sheds): profile it and make the common path cheap'
status: shaving
type: task
priority: 2
created: '2026-10-12T16:04:57Z'
updated: '2026-10-13T00:11:39Z'
parent: yaks-38dc
labels:
- cli
- delta
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
