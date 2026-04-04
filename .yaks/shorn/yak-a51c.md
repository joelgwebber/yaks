---
id: yak-a51c
title: Stateful detail UI (show/hide)
type: task
priority: 1
created: '2026-04-04T22:38:42Z'
updated: '2026-04-04T23:20:28Z'
commit: 35f6794
---

I frequently run short on horizontal screen real estate, because the list is obscured by the
detail pane, and the detail pane often isn't wide enough.

Let's have the detail pane show only when selected, take up relatively more of the terminal (say 2/3 ish), and wrap at least the title and detail text.

We can also ditch the underline selection affordance we've been using when the detail panel's open,
sticking with the regular selection color.
