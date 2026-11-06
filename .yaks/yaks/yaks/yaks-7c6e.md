---
id: yaks-7c6e
title: Holistic key-binding rethink, after the unified-model phases land
status: hairy
type: task
priority: 2
created: '2026-11-03T04:22:32Z'
updated: '2026-11-06T04:50:23Z'
parent: yaks-f677
depends_on:
- yaks-6919
- yaks-0ac1
tags:
- tui
needs: agent
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

---
▸ 2026-11-03T22:06:10Z [delta:starred-batch]

Design pass (delta:starred-batch): [TUI keys: organizing principles](/doks/design/tui-keys.md) states six rules, measures today's table against them, and proposes the changes. Built so far, as uncontroversial bug fixes from your list:

- `J`/`K` in a Wiki page open the next / previous row's page (`2ba1622`).
- Related, under yaks-000b: the list wheel scrolls like the detail; sideways wheel scrolls tables.

The rest waits on the ask below.

---
▸ 2026-11-03T22:06:11Z [delta:starred-batch]

kind: ask
needs: human
---

asked: needs human

Questions from [the keys page](/doks/design/tui-keys.md), with my lean on each:

1. **`h` and leaving the pane.** Lean: `h` (and `←`) only scroll left and never leave the pane; leave with `Esc` (or `Backspace`). `l`/Enter still enter. This fixes the "shot out of the table" problem. OK to drop `h`-to-leave entirely, or keep it only when the row has nothing to scroll?
2. **One family for the whole TUI.** Lean: `vim_mode: false` turns off `hjkl`, `d`/`u`, `g`/`G` and `Ctrl-N`/`Ctrl-P` outside the editor, leaving arrows/PgUp/PgDn/Home/End, and adds `Ctrl-F` find with `F3`/`Shift-F3` next/previous (F3 in both modes). Keep the setting's name (`vim_mode`) or rename it to `keys: vim|standard`?
3. **The case rule** (lowercase looks, Shift changes an existing yak, Ctrl is the farm/app): today's table already obeys it. Lean: adopt it as a written rule plus a test over the table, with no key moves needed.
4. **The `?` popup.** Lean: grouped (Move / Open and follow / Find / Change the yak / View and filter / Farm / App); pressing a listed key runs it; `F1` opens it everywhere, including the editors (where it lists the editor's keys for the active family), and `?` in vim Normal.
5. **Rebinding.** Lean: keep the existing `keys:` overrides by command name and go no further (no editor rebinding; the family choice covers that).
6. **Emacs parity in the editor.** edtui has 164 vim bindings and 46 emacs ones. Lean: a separate yak to audit and fill the emacs set (or the "standard" set, if 2 renames it); not part of this pass.

---
▸ 2026-11-06T04:50:23Z [sched-1]
kind: answer
needs: agent
---

1. Let's drop both h/l for consistency. Enter/escape are perfectly good, and fit comfortably in either vim/emacs modes.
2. Yes to all that. Agreed on `keys: vim|standard`; gives us more options in the future if we need them.
3. I also really like this approach. It should significantly reduce the likelihood of mistakes.
4. Exactly. This will help accessibility and legibility quite a bit.
5. Agreed; let's keep it simple to start. And maybe indefinitely.
6. Also agreed -- we want parity where appropriate, but that will require a careful evaluation, but we can file it for next steps. We should also add basic guidance on key-mappings in AGENTS.md or similar.
