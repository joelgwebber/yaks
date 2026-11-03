---
id: yaks-9c3c
title: 'Design: one TUI model for views, panes, history, search and keys'
status: shaving
type: task
priority: 2
created: '2026-11-03T02:17:24Z'
updated: '2026-11-03T02:17:31Z'
tags:
- tui
- design
---

One design that unifies five TUI yaks, so they land as one coherent model rather than five separate mechanisms:

- yaks-158d: global back/forward navigation.
- yaks-953f: global search context (its design also covers yaks-1bbd).
- yaks-c7ba: a global search affordance.
- yaks-f1e6: the Sheds view made like the list and tree views.
- yaks-0ac1: a rethink of the key bindings.

It also takes in what landed since the first two designs were written: the Wiki tab, the text viewer with link following, peek and its own history, image viewing, and Sheds rebuilt as one row per actor.

The design lives in the repo at [docs/design/tui/model.md](code:docs/design/tui/model.md). The earlier per-yak designs are its references.
