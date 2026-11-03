---
id: yaks-7c6e
title: Holistic key-binding rethink, after the unified-model phases land
status: hairy
type: task
priority: 2
created: '2026-11-03T04:22:32Z'
updated: '2026-11-03T20:58:23Z'
depends_on:
- yaks-6919
- yaks-0ac1
tags:
- tui
---

---
▸ 2026-11-03T12:20:42Z [Joel Webber]

Quoth yaks-6919:

> One wrinkle in lean 8 (which-key) before phase 7 runs. The lean said "a popup after a 600 ms pause on a prefix key". But the keymap has no prefix keys today (every command is one key or one chord), so there is nothing to pause after.
> 
> - **A (my lean):** `?` shows a compact popup at the bottom listing the current context's keys (generated from the table); a second `?` opens today's full help. No timers, testable headlessly.
> - **B:** add a leader key now (`Space`, which collapse uses today, or `,`) with the rarer commands behind it, and the popup after a pause on the leader. That is the holistic rethink's call (yaks-7c6e), so I'd rather not start it here.
> - **C:** skip which-key in phase 7 and leave it to yaks-7c6e.

---
▸ 2026-11-03T20:43:15Z [Joel Webber]

It looks like we went ahead and did the `?` affordance, which I think is a fine place to start.

I have a bunch of other ill-formed thoughts I'd like to address before committing to a design:
- We really need to test whether the emacs/vim keybinding rules are both equally comprehensive;
  especially in the editor, but also throughout.
- Decide whether using std/emacs mode should turn _off_ the weird vim keys (eg, j/k, ctrl-n/p, et al) in most parts of the UI. Probably?
- I'd like to carefully consider all the choices we've made, especially around mutative bindings (eg, (R)eparent, (X)slaughter, (L)abels, and so forth).
- Do we allow keys to be rebound? That's a pretty deep rabbit hole, but TUI users can be persnickety (see also: vim users!)
- The `?` which-key style affordance is
