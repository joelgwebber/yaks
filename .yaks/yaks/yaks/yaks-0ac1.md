---
id: yaks-0ac1
title: Have a big rethink about keyboard bindings
status: shorn
type: task
priority: 3
created: '2026-10-28T12:57:59Z'
updated: '2026-11-03T06:11:22Z'
tags:
- tui
---

At present we have a pretty broad mishmash of vim-like bindings. Even though, at least in theory, we also support emacs-like editor bindings, the rest of the UI is pretty solidly vim-ish.

I don't want to go _crazy_ with configurable key bindings, but I'm pretty sure others won't share my taste on the subject. I'd like to think these through in a few aspects:
- How configurable they should be, and how those options might mesh with the editor keys.
- Ensure consistency in the defaults, eg whether we use ctrl- chords for relatively more or fewer commands.
- What even *constitutes* a command for binding.
- Do we want to allow for a <leader> structure (melodies vs chords?) like modern [n]vim?
- Something more than just `?` help would be good for discoverability.
  Maybe something more contextual and discoverable like nvim-whichkey?
- Ensure coverage of mouse/keyboard affordances where appropriate (eg, you can't click detail editor fields to focus them).
- Invest in whatever refactoring throughout the TUI codebase makes all of this cleaner.
  It'll get nasty if we're not careful and try to bolt it in thoughtlessly.

---
▸ 2026-11-03T02:19:23Z [tui-design]

Partly folded into the unified TUI design, [wiki/design/tui-model.md](/wiki/design/tui-model.md), tracked by yaks-9c3c: §5 "Keys".

- **Command table (phase 2).** A pure refactor that the other phases need: named commands with a context, default keys and a help line. The `?` overlay and the bottom hint are generated from it.
- **The key pass (phase 7).** Settling the collisions the design tabulates, then which-key, user overrides, and mouse bindings through the same table.

Configurability, leader keys and which-key are still open here.

---
▸ 2026-11-03T06:11:22Z [herd-tui]

The command table (phase 2, `642be42e`) and the key pass (phase 7, `c98d7e3c`: one meaning per key for h/v/N/r/q, a `?` keys popup, user `keys:` overrides by command name) landed under yaks-947a. Configurability depth, leader keys and a real which-key move to the holistic rethink, yaks-7c6e.

---
▸ 2026-11-03T06:11:22Z [herd-tui]
kind: move
from: hairy
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
