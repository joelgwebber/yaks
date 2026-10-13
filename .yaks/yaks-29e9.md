---
id: yaks-29e9
title: 'An ask answered in the reference copy (needs: agent there) still reads open in the asking shed'
status: hairy
type: bug
priority: 2
created: '2026-10-13T01:51:06Z'
updated: '2026-10-13T01:51:06Z'
parent: yaks-38dc
labels:
- cli
- delta
- ui
---

Seen 2026-10-13 on yaks-7204: the coordinator asked in its shed, committed, and pushed main to Joel's checkout, so main's copy carried the same open ask. Joel answered it in main (plain `answer`, as the coordinator had told him): main's copy became `needs: agent` + his note, uncommitted. The asking shed's copy stayed `needs: human`, and every surface kept calling it open:
- `yaks sheds` / the Sheds tab in main: the shed shows `⏳` / `1 needs`, not landed: `needs_superseded` (yaks-8814) requires the reference to carry NO `needs:`, and `needs: agent` is exactly "answered, not yet picked up".
- The TUI Inbox in main (yaks-1ea3) lists it as a shed ask, and `a` there would answer it a SECOND time, in the shed.

Fix the rule in one place: a shed's `needs: human` is superseded when the reference holds the shed's ask entry followed by a later entry, whether the reference's `needs:` is cleared OR `agent` (answered). `Snapshot::open_asks` and the landed flag then agree, and the Inbox/`a` stop offering a second answer. Also consider: `inbox --sheds` (same rule?), and whether the coordinating skill should say "do not push an open ask to the primary; tell the human `answer <id>@<shed>`" (the coordinator's mistake here).
Evidence: a test with exactly this case (ref `needs: agent`, ask + answer entries) reads landed; the TUI Inbox in the primary does not list it.
