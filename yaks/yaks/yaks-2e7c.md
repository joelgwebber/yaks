---
id: yaks-2e7c
title: Take the command table mechanism from toque
status: shorn
type: task
priority: 2
created: '2026-11-21T05:22:42Z'
updated: '2026-11-22T17:50:51Z'
tags:
- tui
---

The generic half of src/tui/keymap.rs (Key, Family, Binding, Keymap lookup and overrides, popup and reference rendering) moved to toque's keys feature (toque farm yak-7658) so canon's TUI can share it. yaks keeps its commands, contexts, table and help text; no key, help text or snapshot changes.

#### 2026-11-21T05:22:46Z

*tui-design · kind: move · from: hairy · to: shaving · thread: ksQQ8ThW0PX0SAyxtEBqIr2N1ZLOAHSsupIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T05:29:52Z

*tui-design*

Landed against toque `d0a82db`.

- `src/tui/keymap.rs` keeps the commands, contexts, `BINDINGS`, sections and extras; `Key`, `Family`, `Keymap`, `popup_lines` and the reference layout are toque's. `Cmd` implements `toque::keys::Command`; the filter drawer's state-dependent hint is a `Hint::Custom`.
- `keymap/overrides.rs` is now just where the file lives (`load_user`); toque reads it.
- The rule tests (shared keys, naming, case tier, both families, labels parse) call `toque::keys::rules`.

Evidence: `cargo test` 1271 pass, the 1 failure is the pre-existing `land::...cherry_pick...`. A headless run (`? ? Esc Enter ?` at 120x50) of the old installed binary and the new one is byte-identical (312 lines), covering the hint bar, the popup and the full reference.

#### 2026-11-21T05:29:53Z

*tui-design · kind: move · from: shaving · to: shorn · thread: ksQQ8ThW0PX0SAyxtEBqIr2N1ZLOAHSsupIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-22T17:50:51Z

*herd-files · kind: change · section: none*

No user-facing change.
