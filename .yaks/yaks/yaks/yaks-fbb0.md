---
id: yaks-fbb0
title: 'TUI phase 2: a command table for keys; help and hint generated from it'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:31Z'
updated: '2026-11-03T03:48:20Z'
parent: yaks-947a
tags:
- tui
---

Phase 2 of [the design](/wiki/design/tui-model.md#5-keys) (part of yaks-0ac1): every key binding moves into one command table (named commands, a context, default keys, a help line), dispatched in layers, with the `?` overlay and the bottom hint generated from it. A refactor: no key changes meaning.

---
▸ 2026-11-03T03:47:41Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T03:48:20Z [herd-tui]

Claim (yakherd herd-tui): worker `keys-1` (Sonnet 5.5), from main at `9a8c7b4`.

**Task.** A pure refactor: every key binding of the non-modal surfaces moves into one command table, and the `?` overlay and bottom hint are generated from it. No key changes meaning.

**The table: a new `src/tui/keymap.rs`.**

- `enum Cmd`: one variant per action, each with a stable dotted name (`Cmd::name()`), such as `move.down`, `view.next`, `nav.back`, `yak.state`, `doc.edit` and `farm.sync`. Keep an `ALL` list for tests.
- `enum Ctx`: today's surfaces as they are (`List`, `Detail`, `Viewer`, `Image`, `Wiki`, `Sheds`, `ShedDetail`). Later phases collapse them; do not anticipate that.
- `Binding { ctx, keys, cmd, help, hint }`: keys match a `KeyCode` plus modifiers. A static table, and `lookup(ctx, &KeyEvent) -> Option<Cmd>`.
- `App::run_cmd(cmd, key)`: one match from `Cmd` to the existing `App` methods. An arm body that is more than a call becomes a small method.

**Dispatch.** These handlers become "work out the ctx, look up, run":

- `handle_key_inner` (list and detail arms), in `src/tui/handlers.rs`;
- `handle_sheds_key` and `handle_shed_detail_key`, in `src/tui/sheds_tab.rs`;
- `handle_wiki_key`, in `src/tui/wiki_tab.rs`;
- the non-find part of `handle_attachment_key` (the `find_box` path stays as it is), in `src/tui/handlers.rs`;
- `handle_image_key`, in `src/tui/imageview.rs`.

Modifier arms (Shift-↑↓, Ctrl-N/P, Ctrl-C) are bindings with modifiers. Conditional arms stay one command each, keeping their condition in the body: Esc peeling, `h` (scroll or back), Enter (yank or follow), `S` (bulk or single), `y` (yank or copy).

**Help.**

- `help_hint()` is generated from per-binding hint labels. Its output must be byte-identical to today in every context, because it appears in nearly every snapshot. The dynamic filter hint may stay a special case.
- `help_content()` is generated from the table: a section per context, each binding's key label and help line, plus the explanatory rows that are not bindings (text file, farm links, peek, image, mouse) as a static extras list in the same module. The help overlay's snapshot and the help test in `tests.rs` may change; no other snapshot may.

**Out of scope.**

- Modal overlays: everything else `handle_overlay_key` dispatches (editors, pickers, the drawer, create, help, cmdline, fuzzy, search and find boxes).
- `src/tui/mouse.rs`.
- The outer `handle_key` wrapper and the search-commit toast: worker `notify-1` owns those (yaks-cc16).
- Any change to what a key does.
- Anything else: `yaks ask yaks-fbb0`.

**Evidence.**

- `cargo test --workspace` passes, and `git diff --stat -- src/tui/snapshots/` shows only help-overlay snapshots changed.
- New tests in `keymap.rs`:
  - no two bindings in one ctx share a key;
  - every `Cmd` in `ALL` is bound at least once;
  - every binding with help text appears in `help_content()`;
  - each ctx's generated hint equals today's literal, copied into the test as a fixture.
- Attach the new `?` overlay as a headless frame (`yaks tui --headless`, press `?`).
- Add `keymap.rs` to the `Layout` list in `AGENTS.md`.

**Judge.** herd-tui at landing.
