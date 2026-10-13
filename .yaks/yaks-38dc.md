---
id: yaks-38dc
title: 'Sheds view: show in-flight yak state from other checkouts in the TUI'
status: shaving
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-13T02:49:11Z'
parent: yaks-b5a0
labels:
- ui
- cli
---

Let `yaks tui` show yak state from the other checkouts of this repo (sheds), read-only; the main checkout stays authoritative.

The data layer and the CLI half are built and released (v0.0.14): `yaks sheds`, `yaks changes <shed>`, `yaks inbox --sheds`, `yaks show --sheds`, `yaks answer <id>@<shed>`, `yaks status`, and the global `-C` / `--shed <name>` selector. Discovery is git-only (yaks-c635). What remains of this yak is the TUI over them: the overlay is designed under yaks-7204, opening or re-rooting the TUI on a shed under yaks-7eea, and the landed state under yaks-ed5e.

Principle (Joel, 2026-10-03): every facility that drives a TUI view also exists as a CLI command, and the TUI calls the same library (`src/sheds.rs`), as `farm.rs` serves both today.

Updated 2026-10-10: the original text named "lanes" and a Delta directory scan as a stopgap with pluggable discovery; both are superseded (lane is now shed, discovery is git-only). The notes below are history and keep the word "lane": read it as "shed".

---
▸ 2026-10-03T21:13:29Z [delta-lead]
Feasibility (Delta, 2026-10-03): from one thread, <repo>/.delta/worktrees/*/<repo> is readable and each has its own .git and .yaks (status counts, branch, HEAD, dirty count all obtainable); git worktree list inside a Delta clone shows ONLY that clone, so a Delta lanes view needs the directory scan (or the planned Delta CLI), not git. Sibling repos keep their own .delta (edtui: /Users/joel/src/edtui/.delta/worktrees/<id>). Finished threads' checkouts persist (log O14), so lanes need a liveness hint (dirty/uncommitted yak state, mtime) or they will list stale ones. Related primitive: yaks-70e5 (diff of two refs) gives the 'what changed in that lane' summary.

---
▸ 2026-10-03T22:17:47Z [delta-lead]
Joel (2026-10-03): good with lanes/diff/TUI-lanes; principle to hold: every core facility that drives a TUI view must also exist in a natural CLI form (e.g. yaks lanes, yaks diff <refA> <refB> alongside the lanes view and its diff pane). Design the data layer once, behind the CLI, and have the TUI call it (same as farm.rs serving both today).

---
▸ 2026-10-05T19:48:55Z [delta-lead]
Joel's lanes review (2026-10-04) filed as yaks-c29a (label lanes by actor and in-progress yaks), yaks-1b39 (? when no merge-base; his primary was a shallow repo, fixed with git fetch --unshallow, one pinned commit still a shallow boundary) and yaks-7204 (idea: lane deltas as an overlay across the TUI/CLI). Coordinator delta-lead.

---
▸ 2026-10-06T21:32:29Z [delta-lead]
Terminology decision (Joel, 2026-10-06, via the naming subthread): a "lane" is now a SHED, everywhere, no aliases; `yaks lanes` becomes `yaks sheds`. Text above this note, and all history, keeps the word "lane": read it as "shed". Not decided: "clip" (what a shed has produced) and "barn" (the primary checkout). The rename itself is yaks-dfca; it lands before the baseline fix (yaks-37fa) because both edit the same file.

---
▸ 2026-10-09T20:12:18Z [delta:Delta :Yaks (cont'd)]
Superseded premise (2026-10-09): the "Delta lanes via a directory scan as a stopgap, make discovery pluggable so a Delta CLI can replace it" part of the description no longer applies. Discovery is git-only and needs no Delta CLI (yaks-c635), and the CLI half is `yaks sheds`. What remains of this yak is the TUI view of sheds, designed under yaks-7204 / yaks-ee0a.

---
▸ 2026-10-13T00:11:39Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-13T00:12:39Z [delta-lead]
Spawned 2026-10-13T00:13Z from base 280767f, model anthropic/claude-sonnet-5-5 (pinned): perf-1 (agent c690566f836e424b) on yaks-07fe, needs-1 (agent 370c6f34957d4b31) on yaks-8814; their clones are Delta dirs 53smxwrxenfm and epe9k7eghxv1 (names appear on their first yaks command). delta-lead builds yaks-98f1 in this thread.

---
▸ 2026-10-13T00:28:42Z [delta-lead]
Landed yaks-8814 (needs-1, 503fca3): Delta fast-forwarded this checkout to it; land checks: no MERGE_HEAD, src/sheds.rs and .yaks/yaks-8814.md equal the commit, the diff confined to outstanding_ids + needs_superseded + tests; full cargo test --release green here. land-1's clone is gone, so the original case is covered by its test only. yaks-98f1 (the Sheds tab) shorn by delta-lead; perf-1 (yaks-07fe) still running.

---
▸ 2026-10-13T00:55:08Z [delta-lead]
Landed yaks-07fe (perf-1, aba6d65) as 1c5720c by land.sh (worker base 280767f an ancestor; Delta had left a pending merge; result identical to the computed merge; the merge touched only the worker's 3 files). Checked here:
- `yaks sheds` wall, 5 runs: before (d279fc0) 1.68 1.26 1.31 1.28 1.27 s; after 0.74 0.33 0.31 0.49 0.33 s.
- `sheds`, `sheds --json`, `changes main`, `inbox --sheds` byte-identical (cmp) between the two binaries.
- `yaks tui --headless` first frame (startup scan included): 1.87 s -> 0.80 s.
- Full cargo test --release: 17 suites green, 0 failed.
Follow-up commit (delta-lead): the Sheds tab scans with `sheds::discover_all` (one host read, as `yaks sheds`), `orphaned_pins` becomes test-only (its only caller left), the docs/tui.md scan-time line updated.
Remaining under yaks-38dc: yaks-93fe (worktree-of-a-clone taken for the primary; discovery, now unblocked), yaks-7204 (overlay, next per Joel's plan), yaks-19c3 (activity log).

---
▸ 2026-10-13T01:31:29Z [delta-lead]
Spawned 2026-10-13T01:30Z from base 5392019, model anthropic/claude-sonnet-5-5 (pinned): badge-1 (agent c3d7a3b7d8264841) on yaks-ed6d, inbox-1 (agent 0c5192acae734297) on yaks-1ea3, disco-1 (agent 0cbe15ec6ca34e1a) on yaks-93fe; clones 2fdsqr81jxvy, f34qg6ja8z10, hers5ay2699w (named on their first yaks command).

---
▸ 2026-10-13T02:02:44Z [delta-lead]
Landed (delta-lead, 2026-10-13): yaks-1ea3 (inbox-1, 10710c6) as 6e10823; yaks-ed6d (badge-1, 6257044) as 994d52a; yaks-93fe (disco-1, 09e07f2) as 0767ac2 by land.sh. Gate on the combined tree: cargo test --release, 17 suites green, 569 unit tests. Doc SVGs regenerated once at the end (docshots scene 9 added for the badges and the detail's Sheds section; frames rasterized and looked at).

Incident, for the record: while Delta held a PENDING merge of badge-1's result in this tree (MERGE_HEAD set), I committed a docs follow-up; that commit became a merge with badge-1 as parent but WITHOUT its changes (327 lines dropped), and land.sh then said "landed" because the SHA was already an ancestor. Caught by the external-edits showing badge-1's files removed. Fixed before any push: reset my unpushed commit, merged 6257044 by hand (resolving the yaks skill's TUI sentence, which inbox-1 and badge-1 both rewrote), verified the tree equals `git merge-tree` apart from that resolution and the re-accepted Inbox snapshot, and rebuilt the commit once more with commit-tree when Delta added disco-1's SHA to MERGE_HEAD seconds before the commit (a 3-parent merge claiming disco-1 without its code). Lesson: check `git rev-parse -q --verify MERGE_HEAD` before EVERY commit while workers are finishing, not only at landing.

---
▸ 2026-10-13T02:04:56Z [delta-lead]
Spawned 2026-10-13T02:05Z from base 6b7f5ef, model anthropic/claude-sonnet-5-5 (pinned): ghost-1 (agent 9fbfe3cd7dab40d9) on yaks-9631, fix-1 (agent fec28b15f37248f1) on yaks-29e9.

---
▸ 2026-10-13T02:49:11Z [delta-lead]
Landed yaks-29e9 (fix-1, 8797869) as df3944b and yaks-9631 (ghost-1, ef3233c) as 1bd1e17; each tree checked equal to git merge-tree, no pending merge. Gate after both: cargo test --release 17 suites green, 580 unit tests. Doc SVGs regenerated (tui-shed-only new, the drawer's copies row), rasterized and looked at. Real machine (delta-lead): a scratch git worktree shed named by its actor (ghostcheck: disco-1's fix), a yak created there as a child of yaks-7204 shows as a dim row with 🛖ghostcheck; S and E refuse with 'yaks-3633 is only in shed ghostcheck: read-only here' and write nothing; Enter shows the shed copy under 'only in shed ghostcheck'. Worktree removed. Note: ghost-1's tests were not written first; it showed them failing by disabling the feature (8 of 10 failed), recorded on yaks-9631.
