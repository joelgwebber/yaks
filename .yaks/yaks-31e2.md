---
id: yaks-31e2
title: Attachments attempt to open on wrong path across sheds
status: hairy
type: bug
priority: 2
created: '2026-10-17T13:21:35Z'
updated: '2026-10-17T13:26:59Z'
labels:
- sheds
- tui
- attach
---

When opening an attachment from the TUI, for a yak that exists on a different shed (or if only the _attachment_ exists on said shed), it fails. Because it always attempts to open attachments from the cwd.

---
▸ 2026-10-17T13:26:59Z [Joel Webber]

Let's ensure that there's an equivalent CLI affordance so we make this easy for agents as well.
Related: yaks-991c
