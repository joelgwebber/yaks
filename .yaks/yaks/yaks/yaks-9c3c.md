---
id: yaks-9c3c
title: 'Design: one TUI model for views, panes, history, search and keys'
status: shorn
type: task
priority: 2
created: '2026-11-03T02:17:24Z'
updated: '2026-11-03T03:17:06Z'
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

The design is the farm page [wiki/design/tui-model.md](/wiki/design/tui-model.md). The earlier per-yak designs are its references.

---
▸ 2026-11-03T02:19:24Z [tui-design]

Drafted [wiki/design/tui-model.md](/wiki/design/tui-model.md) and added a note to each folded yak (yaks-158d, yaks-953f, yaks-1bbd, yaks-c7ba, yaks-f1e6, yaks-0ac1) pointing back to it.

Facts checked before writing:

- A headless run of the current build shows the stale `filter:` toast.
- The same run shows the text viewer swallowing `U` and `Tab`: the overlay stays open and nothing happens.
- The code shows three back stacks: `nav_back`/`nav_fwd`, `viewer_back`/`viewer_fwd`, and `LineNav.back`/`fwd`.
- The code shows that `/` matches different fields on the Wiki tab than on the yak tabs.

Open for Joel:

- the detached pane, which revisits decision 5;
- the query matching rule;
- pages in Starred and Recent;
- putting the command table second.

---
▸ 2026-11-03T03:17:06Z [tui-design]

Design agreed by Joel, 2026-11-02: every open question resolved as leaned (decisions 5 to 8 on the page). The page is [wiki/design/tui-model.md](/wiki/design/tui-model.md). The questions on yaks-158d and yaks-953f are answered there and their `needs` cleared.

Implementation is the page's seven phases, carried by the folded yaks (yaks-953f phase 1 first, then the command table under yaks-0ac1).

---
▸ 2026-11-03T03:17:06Z [tui-design]
kind: move
from: shaving
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
