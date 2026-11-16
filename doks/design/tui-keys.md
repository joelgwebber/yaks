---
type: topic
title: 'TUI keys: organizing principles'
tags:
- tui
- design
---

# TUI keys: organizing principles

A proposal for yaks-7c6e, after the unified-model phases ([TUI design](/doks/design/tui-model.md)) landed the command table (`src/tui/keymap.rs`), the `?` popup and `keys:` overrides. It states the rules a binding must follow, measures today's table against them, and lists what would change. Decisions are asked on yaks-7c6e; nothing here is built yet.

## The rules

1. **One key, one intent, everywhere.** A key means the same thing in every context; only *what* it acts on changes with focus (movement moves whatever has focus). Already the stated rule of the design page; this page applies it to the cases that still break it.
2. **Case is a tier, not a coin toss.**
   - lowercase: look and move (navigate, open, find, copy). Safe to mash.
   - Shift: change an existing yak (state, fields, structure, comments). No lowercase key changes an existing yak. (Creating one, `c`/`C`, and answering, `a`, add rather than change, and stay lowercase; a few read-only commands sit on Shift for want of a letter, which is harmless.)
   - Ctrl: the app and the farm (quit, commit, sync, rescan), plus the editor's own keys.
   A reader can tell a key is safe from its case alone.
3. **Destructive means confirmed, and hard to hit.** Slaughter, a bulk state change and discarding an edit always confirm. No destructive action sits on a key adjacent to a common navigation key.
4. **Two families, chosen once.** `vim_mode` picks a family for the whole TUI, not just the editor:
   - vim: `hjkl`, `d`/`u`, `g`/`G`, `/` `n` `N`, `v`, `:w`.
   - standard (today's "emacs"): arrows, PgUp/PgDn, Home/End, `Ctrl-F`/`/` find, `F3`/`Shift-F3` next/previous, Shift-arrows select.
   Keys that belong only to the other family are off: in standard mode `j`/`k`/`h`/`l` and `Ctrl-N`/`Ctrl-P` do nothing outside the editor. Keys both families share (Tab, Enter, Esc, arrows, the Shift tier) work in both.
5. **Enter goes in, Esc comes out.** Enter opens the row, follows the link, commits a one-line field; Esc peels one layer (selection, then search, then pane). `h`/`l` (vim) are *movement*, not pane switching: `l` scrolls right, `h` scrolls left and never leaves the pane.
6. **Discoverable by construction.** Every binding is in the table, so in `?`, the bottom hint and the docs. `?` (and `F1`, which works in every mode, in the editor too) opens the context's popup, grouped; pressing a listed key in the popup runs it.

## Today's table against the rules

- **Rule 2 already holds.** Every change to an existing yak sits on Shift: `S` state, `P` priority, `T` type, `L` tags, `D` dependency, `R` reparent, `H` herd, `X` slaughter, `E` edit, `M` comment, `Q` ask, `A` attach. `c`/`C` (create), `a` (answer), `m` (mark) and `*` (star) are lowercase and add or mark rather than change. `F`, `V`, `B`, `O` are read-only on Shift. The one lowercase change is `r` in the detail (rename an attachment, which rewrites links farm-wide); it opens a prompt first.
- **Rule 4 is not there.** `vim_mode` only switches the editor. The list, detail and viewer answer `hjkl`, `d`/`u`, `Ctrl-N`/`Ctrl-P` in both modes, and standard mode has no find-next key besides `n`.
- **Rule 5 breaks on `h`.** In the detail and viewer, `h` scrolls a wide block left and, at offset 0, leaves the pane. Joel's report: scrolling left through a table shoots out of the pane.
- **`J`/`K` were missing in a Wiki page;** fixed (`2ba1622`, a page's `J`/`K` open the next row's page).
- **Rule 6:** the `?` popup is flat (one list per context), the editor has no popup, and keys pressed in the popup do not run.

## What would change

| Today | Proposed | Why |
|---|---|---|
| `h` at offset 0 leaves the pane | `h` only scrolls; leave with `Esc` (or `Backspace`) | rule 5; the table-scrolling report |
| `r` renames an attachment (lowercase) | `Shift-R` is taken (reparent), so keep `r` behind its prompt, and make the prompt the confirmation | rule 2, by the prompt rather than the case |
| `vim_mode` editor-only | one `keys: vim` / `keys: standard` setting for the whole TUI (keep `vim_mode` as its alias) | rule 4 |
| standard mode: `n`/`N` only | add `F3`/`Shift-F3` (both modes), `Ctrl-F` opens find in standard mode | rule 4 |
| flat `?` popup | grouped: Move, Open and follow, Find, Change the yak (Shift), View and filter, Farm (Ctrl), App | rule 6 |
| no editor popup | `F1` in every editor (and `?` in vim Normal) shows the editor's keys for the active family | rule 6 |
| popup is read-only | a key pressed in the popup runs it and closes the popup | rule 6 |

## Rebinding

`keys:` overrides already exist (`src/tui/keymap/overrides.rs`), by command name. The proposal adds nothing there beyond making the family a setting. Rebinding the *editor's* keys stays out: edtui owns them, and a family choice covers the common complaint.

## A check that keeps it true

A test over the table enforces rules 2 and 4: every command marked as a mutation is bound to a Shift key or confirms; no binding in standard mode uses a vim-only key. A new key that breaks a rule fails the build rather than waiting for a reader to notice.

## Built

yaks-7c6e built this page's proposal (2026-11-07), with Joel's answers: `h`/`l` never move between list and pane; `keys: vim|standard` in the user config picks the family for the whole TUI (`vim_mode` stays an alias); the case rule and "every command stays bound in standard mode" are tests over the table; the `?` popup is grouped, runs a listed key, and `F1` opens it everywhere, editors included; `{`/`}` change views. `AGENTS.md` has the key-binding rules for contributors; user-facing behaviour is in `docs/tui.md`. Editor parity across families is yaks-4736.
