---
id: yaks-683f
title: Save incomplete edits of yaks and descriptions/comments
status: shaving
type: feature
priority: 3
created: '2026-08-23T02:49:48Z'
updated: '2026-11-10T17:49:46Z'
tags:
- tui
needs: agent
---

Preserve a draft when an edit form or comment is dismissed or interrupted, so in-progress text is not lost.

---
▸ 2026-08-26T13:02:05Z

Related: yaks-275f (confirm-on-cancel for dirty yaks). Both address losing work when an edit is dismissed; coordinate the designs.

[683f-design](yaks-683f/683f-design.md)

---
▸ 2026-09-23T21:39:11Z [design-scout]

Design proposal attached

---
▸ 2026-09-23T21:39:11Z [design-scout]

Design attached (683f-design.md). Decisions:
1. Dirty cancel: replace 275f's y/N with 'k=keep draft (default) / d=discard / Esc=back', or drop the prompt and always keep silently? Lean: 3-way prompt, keep default.
2. Restore: auto-restore with a 'draft' badge + Ctrl-R to revert, vs a 'restore draft?' prompt? Lean: auto-restore.
3. Phase 1 saves on dismiss/quit only (autosave via the 250ms loop in P2), or autosave from day one? Lean: dismiss/quit first.
4. Mid-edit external changes clobbered on commit: in scope or separate yak? Lean: separate; 683f only warns on restore when base 'updated' differs. Drafts stored at `~/.config/yaks/<slug>/drafts/<key>.json`, keys `edit:<id>/comment:<id>/create:<parent|root>`.

---
▸ 2026-11-06T04:58:33Z [sched-1]
kind: answer
needs: agent
---

1. I like the three-way prompt idea.
2. I'm good to start by trying auto-restore.
3. Yeah, we can start there and see how it goes. I originally wanted this becauuse of crash bugs, but we haven't had that problem in a while.
4. SGTM.

---
▸ 2026-11-10T17:49:46Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
