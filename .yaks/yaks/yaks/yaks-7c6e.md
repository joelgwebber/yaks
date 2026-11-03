---
id: yaks-7c6e
title: Holistic key-binding rethink, after the unified-model phases land
status: hairy
type: task
priority: 2
created: '2026-11-03T04:22:32Z'
updated: '2026-11-03T21:13:54Z'
parent: yaks-f677
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
- The `?` which-key style affordance is really useful. We should extend it to the editor (perhaps `?` in vim-normal mode, and always-available `F1`?). It would also be nice if pressing any of those keys from within the `?` menu immediately triggered the action (this is really helpful in which-key on vim).
- I'm no longer sure that h/l used for moving in/out of the detail panel is still paying its freight over enter/escape. It's particularly annoying when using `h` to scroll left in an overflowing table, and you shoot off the left end, escaping the detail view.
- The `?` popup would be easier to parse if organized into groups. Open to ideas on good groupings.
- J/K doesn't work in the wiki doc details, like it does in yaks.

Probably lots more cases I'm not thinking of. Please explore the idea of organizing principles for the keyboard controls as a whole, so we can get a bit more consistency. We should probably think about yaks-000b in this context as well.
