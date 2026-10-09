---
id: yaks-38dc
title: 'Lanes view: show in-flight yak state from sibling checkouts'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-09T20:12:18Z'
parent: yaks-b5a0
labels:
- ui
- cli
---

Let yaks tui (and list/show) overlay yak state from other checkouts of the same repo: git lanes via git worktree list; Delta lanes via <repo>/.delta/worktrees/<id>/<repo>/.yaks as a stopgap (opaque ids, internal layout, local runs only). Read-only; the main checkout stays authoritative. Make discovery pluggable so the planned Delta CLI ('connect any agent to Delta threads and worktrees from the command line') can replace the directory scan. Relates to yaks-5f12 (live cross-worktree sharing).

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
