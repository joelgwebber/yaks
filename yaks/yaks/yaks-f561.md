---
id: yaks-f561
title: 'TUI: a Due tab and an overdue chip'
status: shorn
type: feature
priority: 2
created: '2026-11-04T22:31:56Z'
updated: '2026-11-04T23:41:33Z'
parent: yaks-7fa6
tags:
- tui
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)).

- A **Due** view beside Inbox: yaks with `due:`, overdue first then soonest, same rows and keys as the list.
- A status-bar chip `⏰N` (N overdue) next to the unpublished chip, hidden at 0 and shortened like it when narrow.
- Once the routing yak lands, check that a yak regrown by the TUI's tick shows in the Inbox with its badge.

Done when: snapshots, `docs/tui.md`, and the `?` help generated from the table.

---
▸ 2026-11-04T23:27:45Z [sched-1]
kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[yaks-f561-frames](yaks-f561/yaks-f561-frames.md)

---
▸ 2026-11-04T23:41:32Z [sched-1]

Done on branch `scheduling`.

- **Due view.** `view::due_view()`: every open yak with a readable `due:`, flat, soonest first, so the overdue ones lead (`FilterSpec::due_only`, `SortField::Due`). It is unpinned, like Inbox and All, reached from the view picker (`` ` ``). I made it unpinned after finding the pinned version pushed the tab strip over its width and cut off the query chip (an existing test caught it); the overdue chip below is what says when to open it. A saved views file from before it gets Due next to Inbox (`reconcile`, tested). `due_only` is saved with a view and carried through the drawer.
- **Overdue chip.** The tab row's right-hand chip now leads with `⏰ N overdue` in red (open yaks past `due:`, counted over the whole farm so every tab says the same), then the dim git segments. It shortens to `⏰N` like the git ones and drops out when there is no room. `⏰` (U+23F0) is a wide glyph that `disp_width` counted as 1, which would have mis-sized the chip: added it beside the other wide ones.
- **Evidence.** Tests for the view's rows and order (a snapshot), the chip (counts open only, same on another tab, full and short forms, none when nothing is late), and the saved-views migration. Updated, after reading each old-versus-new diff: the view-picker snapshot (one new row) and the tests that hard-coded view indexes. Full `cargo test` green (979 + 31 suites); clippy per-file counts unchanged. Looking at the real binary with `yaks tui --headless`: the view picker lists `Due (4)`, selecting it shows the three dated yaks soonest first, and the chip reads `⏰2 unpub`. Frames attached (`yaks-f561-frames.md`).
- **Docs.** `docs/tui.md`: the view list and the right-hand chip (order, colour, short forms).

Decision: unpinned (above). Decision: the chip counts the whole farm, not the active view, so it does not change under you when you switch tabs.

Not done: the notice that a woken recurring yak is in the Inbox was already added with yaks-a041 (`regrew X (awaiting agent)`), so there is nothing more to do for that bullet.

---
▸ 2026-11-04T23:41:33Z [sched-1]
kind: move
from: shaving
to: shorn
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
