---
id: yaks-0ac1
title: Have a big rethink about keyboard bindings
status: hairy
type: task
priority: 3
created: '2026-10-28T12:57:59Z'
updated: '2026-10-28T12:57:59Z'
labels:
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
