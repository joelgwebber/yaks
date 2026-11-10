---
id: yaks-ca57
title: Auto-complete for dok-links
status: shaving
type: feature
priority: 3
created: '2026-11-10T17:37:34Z'
updated: '2026-11-10T17:51:42Z'
tags:
- doks
- tui
---

Just like we do when typing `[prefix]-<tab>` in a detail edit.

---
▸ 2026-11-10T17:49:47Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:31Z [herd-unlock]

Claim (yakherd herd-unlock): worker `doklink-1`, Sonnet 5.5.

## Scope

In the TUI's multi-line editors, Tab after a dok-link token opens a picker over the farm's doks and inserts the chosen path, as Tab after `<prefix>-` does for yak ids (`completion_context` in `src/tui/editor.rs`, `open_ref_picker` and the `EditKey::Complete` paths in `src/tui/handlers.rs`). Trigger: a token starting `/doks/` or `/<herd>/doks/` (the farm-root link form), seed = what follows. Inserted text: the full farm-root path. Reuse the existing picker overlay; the yak completion must behave exactly as before. Docs: `docs/tui.md`, the keymap `EXTRAS` row for Tab completion.

## Out of scope

Heading anchors (`#...`), relative links, sources.

## Evidence

Tests: Tab after `/doks/des` opens the picker seeded `des` and inserting replaces the token; `yaks-` completion unchanged. A toque text frame of the picker open over an editor, attached. Gate `cargo test --workspace`. Judge: the yakherd.

---
▸ 2026-11-10T17:51:42Z [doklink-1]

Start: Tue Nov 10 17:51:39 UTC 2026

- farm path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-ca57.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/9qjdhqw40bqc/yaks
