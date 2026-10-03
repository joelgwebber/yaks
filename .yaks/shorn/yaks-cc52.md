---
id: yaks-cc52
title: Following a link to a yak outside any pinned view shows the previous yak
type: bug
priority: 3
created: '2026-09-23T21:40:23Z'
updated: '2026-10-03T21:16:23Z'
labels:
- ui
---

Found by the yaks-158d design scout: follow_link to a yak with no pinned status view (e.g. a dead yak) leaves the cursor unchanged; the detail pane renders rows()[cursor], so it keeps showing the previous yak under a '→ id' notice. yaks-28b4 guards back/forward but not follow_link. yaks-63f3 split goto_task out of follow_link — check whether that path has the same gap. See yaks-158d's design attachment (P3).

---
▸ 2026-09-23T21:43:01Z [coordinator]
Update after merge: yaks-63f3's goto_task (now used by follow_link) checks whether the cursor landed on the target and otherwise shows '<id> isn't shown in any view'. That may already cover this; confirm by following a link to a dead yak before fixing.

---
▸ 2026-10-03T21:14:11Z [delta-lead]
TRIAL 1 (yaks-fe33), worker B 'trial1-sonnet' (claude-sonnet-5-5). Claim deliberately NOT committed (tests whether the worker's snapshot includes uncommitted claims, log O3). Scope: follow_link/goto_task in src/tui/detail_nav.rs and tests in src/tui/tests.rs. First confirm the bug still reproduces (the 63f3 note suggests it may be fixed) by driving the headless TUI (toque) and attach the frame. Gate: cargo test -p yaks. Judge: coordinator re-runs the gate and reads the attached frame.

---
▸ 2026-10-03T21:15:17Z [trial1-sonnet]
Start 2026-10-03T21:14:41Z. ls .yaks/*/yaks-cc52.md => .yaks/shaving/yaks-cc52.md (status dir: shaving; git shows hairy/ copy deleted + shaving/ untracked, i.e. claim is uncommitted but present in my worktree). target/ did NOT exist before build; cargo build --release took 16.97s (finished 21:15:02Z).

![follow-dead-link-after](artifacts/yaks-cc52/follow-dead-link-after.txt)

![follow-dead-link-guard-removed](artifacts/yaks-cc52/follow-dead-link-guard-removed.txt)

---
▸ 2026-10-03T21:16:23Z [trial1-sonnet]
FINDING: bug does NOT reproduce on current code; already fixed by yaks-63f3's goto_task guard (after open_task_in_detail, if selected_id() != target it sets notification '<id> isn't shown in any view' and returns before pushing nav history or the '→ id' notice). Repro: a0 (hairy) body 'see d0 for the old plan', d0 Dead; no pinned Dead view; Enter (detail), Tab (link), Enter (follow) at 72x16. Observed frame (attached follow-dead-link-after.txt): notice row reads "d0 isn't shown in any view", detail still shows a0 (cursor can't move), no '→ d0'. Mutation check: with the guard disabled ('if false && ...') the new test fails with left: Some("→ d0") right: Some("d0 isn't shown in any view") and the frame shows '→ d0' over a0's detail = the reported bug (attached follow-dead-link-guard-removed.txt). Guard restored (git diff of detail_nav.rs is empty). No product code changed.

---
▸ 2026-10-03T21:16:23Z [trial1-sonnet]
SUMMARY: added regression test follow_link_to_yak_outside_any_view_says_so in src/tui/tests.rs (asserts notification, rendered frame, no '→ d0', empty nav_back). Gate cargo test -p yaks: unittests 322 passed/0 failed/3 ignored; tests/cli.rs 28 passed. No snapshot changes. Shearing as already fixed.
