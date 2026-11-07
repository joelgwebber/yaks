---
id: yaks-7c6e
title: Holistic key-binding rethink, after the unified-model phases land
status: shaving
type: task
priority: 2
created: '2026-11-03T04:22:32Z'
updated: '2026-11-07T18:16:29Z'
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

---
▸ 2026-11-06T22:46:52Z [sched-1]

One other idea: It would be really helpful to have something like {/} allow you to change tabs while in a detail. This arguably congruent with J/K.

---
▸ 2026-11-07T17:04:27Z [herd-stars]
kind: pickup
---

picked up

herd-stars: implementing the six answered points plus the {/} tab idea, after the starred TUI batch lands (they all touch the keymap).

---
▸ 2026-11-07T17:40:32Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-keys`, to start once the starred TUI batch (yaks-a004, yaks-255d, yaks-000b, yaks-e5f9) has landed, since every one of them adds bindings. Spec from Joel's answers (2026-11-06) and [the keys page](/doks/design/tui-keys.md):

## 1. `h`/`l` never move between list and pane

- Detail pane and text viewer: `h`/`←` and `l`/`→` only scroll a code block or table sideways; at offset 0 `h` does nothing. `Esc` leaves (peel), `Enter` goes in.
- List: `Enter` opens the pane. On tree rows (yaks, sheds) `h`/`←` stays `tree.left` and `l`/`→` becomes `tree.right` (expand a collapsed row, else move to its first child). On document rows `l`/`→` do nothing.
- Hints: `l:detail` becomes `Enter:detail`; `h:list` becomes `Esc:list`.

## 2. One key family for the whole TUI

- The family is a personal setting, so it lives in the user config (`$XDG_CONFIG_HOME/yaks/config.yaml`), where `keys:` already holds overrides. `keys: vim` or `keys: standard` (a scalar) picks the family; the map form keeps the overrides and may hold `family: standard` (not a command name, so no clash). Legacy `vim_mode:` is an alias: user `keys` family, else user `vim_mode`, else farm `vim_mode`, else vim.
- Standard: outside the editor, `h` `j` `k` `l`, `d`/`u`, `g`/`G` and `Ctrl-N`/`Ctrl-P` are off; arrows, PgUp/PgDn, Home/End do the work (add them wherever a context has only the vim key). `Ctrl-F` opens find. The editor runs edtui's emacs mode.
- Both families: `F3`/`Shift-F3` find next/previous; `Ctrl-↓`/`Ctrl-↑` next/previous content block (beside `Ctrl-N`/`Ctrl-P`).
- A test over the table: in standard mode every context still has a binding for every command it had in vim mode.

## 3. The case rule, as a test

Lowercase looks and moves; Shift changes an existing yak; Ctrl is the app and farm. A test over `BINDINGS` lists the commands that change an existing yak and fails if one is on a lowercase key, with the documented exceptions (`r` rename behind its prompt; `c`/`C`, `a`, `m`, `*` add or mark).

## 4. The `?` popup

- Grouped: Move, Open and follow, Find, Change the yak, View and filter, Farm, App (a group per `Cmd`, from the table).
- A key pressed in the popup that is listed there runs it and closes the popup. `?` again still opens the full reference; any other key closes.
- `F1` opens it everywhere, including every editor (the form, the in-place editor, the dok editor), where it lists the editor's keys for the active family (a hand table, as `EXTRAS`, since edtui owns them); `?` in vim Normal mode too. A listed key pressed there closes the popup and goes to the editor.

## 5. `{` / `}` change views

`}` next view and `{` previous everywhere (list, pane, viewer), as `J`/`K` step rows: in the pane, the pane stays open on the new view's selected row.

## 6. Guidance and follow-up

- `AGENTS.md`: a short "Key bindings" paragraph under the docs rule: the rules above, and that the tests enforce two of them.
- `docs/tui.md` (keys section, every key table, Customising keys), `README.md` (its `vim_mode` line), the `?` help.
- Emacs/standard parity in the editor is yaks-4736, not this.

**Evidence.** The rule tests (fail first where a binding breaks them), a test per numbered item, the gate, headless frames of the grouped popup in the list and in an editor (`F1`), attached. **Judge:** herd-stars at landing; Joel for the feel.

---
▸ 2026-11-07T17:46:47Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:51:18Z [stars-keys]

Started by worker `stars-keys`.

- `date -u`: Sat Nov  7 17:47:16 UTC 2026
- `yaks path yaks-7c6e`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-7c6e.md
- checkout top-level: /Users/joel/src/yaks/.delta/worktrees/y2d4pv9j3yrk/yaks
- base commit: f0dafed7ca3644ad4c0601f0bfb7bb618c047453

Read the claim note, Joel's answers and the design dok. Plan: one `Family` (vim|standard) in the `Keymap`, replacing the `App.editor_vim` field (it becomes a method over the keymap so there is a single source of truth); `h`/`l` leave only `←`/`→`-style scrolling and `tree.left`/`tree.right`; case-rule and family tests over `BINDINGS`; grouped `?` popup with run-on-press; `F1` popup over editors from a hand table; `{`/`}` views everywhere; docs.

---
▸ 2026-11-07T18:03:46Z [stars-keys]

Progress (17:47 start, now about 19:00 wall clock): items 1, 2, 4 (the popup and run-on-press), 5 and the rules are in; the existing tests are migrated and the suite is green apart from snapshot acceptance, now done. Still to write: the rule tests, a test per item, the F1/editor popup tests, docs.

Friction: the machine's data volume was at 100% (137 MiB free) when I first built, from the other checkouts' `target/` dirs. I cleaned only my own `target/` and build with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0` (environment variables, no Cargo.toml edit); that is the only reason the gate fits.

[popup-list-vim](yaks-7c6e/popup-list-vim.txt "The grouped ? popup on the list (vim family, 110x24): Move / Open and follow / Find / Change the yak / View and filter / Farm / App, a listed key runs it")

[popup-editor-vim-normal](yaks-7c6e/popup-editor-vim-normal.txt "F1 over the in-place comment editor (vim Normal): the editor's own keys, grouped; a listed key goes on to the editor")

[popup-editor-standard](yaks-7c6e/popup-editor-standard.txt "F1 over the same editor in the standard family: the emacs-style keys")

[hint-list-standard](yaks-7c6e/hint-list-standard.txt "The list in standard mode: the bottom hint names the arrows (↑/↓:move), Enter:detail")

[popup-list-standard](yaks-7c6e/popup-list-standard.txt "The ? popup on the list in standard mode: no h j k l, d/u, g/G; Ctrl-F find, Home/End, F3")

[brace-in-pane](yaks-7c6e/brace-in-pane.txt "} with the pane open: the pane stays open, on the new view's (Shaving) first row")

---
▸ 2026-11-07T18:15:10Z [stars-keys]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-07T18:16:29Z [stars-keys]

Decision: a Ctrl chord no longer falls through to the plain key it carries (`Key::matches`). Before, `Ctrl-N` in the viewer matched `n` (find next) and `Ctrl-P` in the pane matched `p` (peek), so "Ctrl-N is off in standard mode" could not be true. A plain binding now ignores Shift (an upper-case letter) and AltGr (Ctrl+Alt), but not a lone Ctrl. Rejected: leaving the quirk and special-casing the two keys. The old test that pinned the quirk (`Ctrl-N` in the viewer is `find.next`) now says `None`.

Decision: half-page and page keys. `d`/`u` are vim-only and the test needs a standard key, so `Ctrl-D`/`Ctrl-U` are bound for `move.half_down`/`half_up` in the list, pane and viewer in BOTH families (they are `d`/`u` in real vim too). The pane gets `PgDn`/`PgUp` (`move.page_down/up`, full page) and `Home`/`End`; the viewer's `PgDn`/`PgUp` change from half-page to full-page, as in the list, so one key means one thing. Rejected: a standard-only pair (`Alt-V`/`Ctrl-V`) or letting half-page lapse in standard mode (the spec's test forbids it).

Decision: `pane.back` is removed, with no alias. `h`/`←` in the pane and viewer are the new `pane.scroll_left` (pair of `pane.scroll_right`), `Esc` is `pane.peel`; in the list `l`/`→` are the new `tree.right` and only `Enter` is `pane.open`. A user `keys:` override of `pane.back` is reported as an unknown command at startup (documented in `docs/tui.md`), as the 0.0.32 wiki rename was.

Decision: `view.next`/`view.prev` are now bound in the pane, the text viewer and the image viewer (`}`/`{`), as well as in the list (`Tab`/`]`/`}`). An override still replaces a command's keys in every context it is bound in, so `view.next: [Ctrl-N]` would now clash with `block.next` in the pane and be dropped; the doc example and tests use `Ctrl-L` instead. Rejected: a second command name for the pane (two names for one intent).

Decision: the popup. A listed key in the list/pane popup runs and closes it, except `Esc`, which only closes it (it is bound to `view.revert`/`pane.peel`, and clearing a search because you dismissed a popup would surprise); `q` is listed, so it quits, as the spec reads. `?`/`F1` again opens the full reference. Over an editor, `F1` again only closes: the full reference is an `Overlay` and would replace the editor under it. The popup is the flag `App::keys_popup`, not an `Overlay` variant (`Overlay::Keys` is gone), so the editor stays in place and comes back untouched; it is shown in the headless state header as `popup=keys`.

Decision: `F1` covers every overlay that has an edtui field or chip rows: the comment/in-place/dok editors, the create/edit form (text rows and chip rows), the drawer, the search and find boxes and the picker queries. Not the single-key prompts (`Pick`, `Confirm`), the view manager or the full reference: they name their keys on the bottom line and have no editor. The chip rows get a small `rows_groups` table. `?` opens it only in a vim editor's Normal mode (in Insert and in standard it is a character). The hand tables (`src/tui/keymap/editor.rs`) list edtui's common keys per family, mode and single-line-or-not, not every binding (emacs parity is yaks-4736); a test checks that every listed key parses.

Decision: one source of truth for the family. `App.editor_vim: bool` is gone; `App::editor_vim()` reads `Keymap::family`, which `Keymap::load_user(farm_default)` resolves as user `keys` family > user `vim_mode` > farm `vim_mode` > vim. The modal overlays that are not in the table also follow the family where they used `hjkl`: the form's and drawer's chip rows, the view manager and the full reference. Rejected: keeping a second bool for the editors.

Decision: a user override is the user's word, so the family does not filter it (`move.down: [j]` works in standard mode). `Ctrl-F` is standard-only (`STANDARD_ONLY`), so in vim `Ctrl-F` is nothing; in standard it opens search/find. Hint labels that name vim keys use `Hint::Family` (`j/k:move` becomes `↑/↓:move`); the viewer and image hint lines, which are fixed text, switch the same way.

Decision: the case-rule test is driven by `Cmd::mutates()`, an exhaustive `match` (a new command cannot be added without choosing), not a hand list in the test. Exceptions are in the test with a reason each: `c` (new yak, new dok), `a`, `m`, `*`, `r`. A stale exception (no lowercase key any more) fails too. The popup's groups come the same way from the exhaustive `Cmd::group()`; where a command could sit in two groups I chose: select/copy under Move, links under Move, go-to and references under Find, quit/help under App, rescan under Farm.

Decision: `{`/`}` with the pane open is one jump (`switch_tab`): the new view's first row is shown in the pane from its top (a yak or shed: the detail; a dok: the viewer; an empty view: back to the list, nothing to show). A dok or image open from a yak's detail is closed by the view switch, as any view switch does. Rejected: keeping a dok open across views (it belongs to the view it was opened from, `set_view` already says so).

Decision: the bottom-hint snapshot changes, all explained by items 1 and 2. List/Doks/Sheds hints `l:detail` is now `Enter:detail` (every `Tab:view ...` snapshot: tree_with_ghost_family, family_indicator_on_tab_row, starred_marker_and_tab_bar, collapsed_root_hides_children, list_scroll_up_moves_viewport_minimally, the memo chip_* and status_notification_* frames, search_commit_then_tab_no_toast, the_due_view..., an_overdue_date..., sheds_tab_list, tests/snapshots/cli__headless_session). Pane hints `h:list` is now `Esc:list` (detail_shows_children_and_links, detail_header_lists_prs_as_links, goto_jumped_to_detail, comment_header_with_metadata, sheds_tab_detail, sheds_tab_yak_detail). The viewer line loses `/h` (`Esc/h close` is `Esc close`: doks_tab_dok). The full reference (help_overlay, memo help_overlay_shows_the_farm_keys) gains the `}`/`{`, `Ctrl-D`, `Ctrl-↓` keys and the pane's `Home`/`End`/`PgDn`; `keys_popup` is the grouped popup. No frame changed in any other way (I read each diff).

Decision: docs/assets SVGs are not regenerated. `cargo test -p yaks docshots -- --ignored` rewrites 30 files, mostly drift that was already there (the checked-in frames still show the pre-Doks tab strip and an older SVG opacity style); I reverted all of it rather than commit unrelated drift. Only the scene source changed: `tui-sheds-detail` pressed `l` to open the shed, which is `Enter` now. The hint text inside the committed SVGs (`l:detail`, `h:list`) is therefore still the old one until someone regenerates them in one commit.
