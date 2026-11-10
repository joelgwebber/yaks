---
id: yaks-affb
title: Copy dok path with `y`
status: shaving
type: feature
priority: 3
created: '2026-11-10T17:36:26Z'
updated: '2026-11-10T17:51:32Z'
tags:
- doks
- tui
---

Just like on yak list & details. At present, `y` just copies the current line of a dok. It should require that the line be selected first (like yaks) in order to copy it.

---
▸ 2026-11-10T17:49:47Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:30Z [herd-unlock]

Claim (yakherd herd-unlock): worker `dokcopy-1`, Sonnet 5.5.

## Scope

In the text viewer (a dok open in the right pane), `y` with no selection copies the dok's path (farm-root form, `/doks/...`), as `y` does on a yak; with a selection, it copies the selection. Today it copies the current line. Find why the current line counts (the `SelectYank` path in the `Viewer` context, `src/tui/keymap.rs` and its handler) and fix the rule, not the key. Help line and hint, `docs/tui.md`.

## Out of scope

The Doks tab row `y` (already copies the path), other contexts.

## Evidence

A test: viewer open, no selection, `y` puts the path on the (test) clipboard; with a `v` selection it copies the selection. Gate `cargo test --workspace`. Judge: the yakherd.

---
▸ 2026-11-10T17:51:32Z [dokcopy-1]

Start: 2026-11-10T17:51:29Z

- yaks path yaks-affb: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-affb.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/am99yre814ns/yaks
