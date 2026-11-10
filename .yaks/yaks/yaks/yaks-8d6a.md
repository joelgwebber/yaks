---
id: yaks-8d6a
title: Label auto-completion
status: shaving
type: feature
priority: 3
created: '2026-08-26T13:24:15Z'
updated: '2026-11-10T21:12:25Z'
parent: yaks-f04d
depends_on:
- yaks-b2ed
tags:
- labels
- tui
---

Once you have predefined labels, we can auto-complete them in the UI.

---
▸ 2026-11-10T21:11:51Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:12:25Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `tagcomplete-1`, Sonnet 5.5. (Depends on yaks-b2ed, which Joel agreed is covered by yaks-5656's picker.)

## Scope

In the create/edit form's `tags` field, `Tab` on a partial tag opens the existing fuzzy picker (as `Tab` after `yaks-` does in an editor: `open_ref_picker`, `FuzzyPick`) over the farm's tags: the known tags from `schemas/yak.md` `tags:` (with their descriptions as the row text) first, then every tag in use, by count. Enter replaces the partial tag at the cursor. Today `Tab` in that field moves to the next field: keep that when the cursor is not on a partial tag (empty, or right after a comma/space). `src/tui/create.rs`, `src/tui/handlers.rs` (the form's Tab path), `src/tui/fuzzy.rs`; help (`src/tui/keymap/editor.rs` row for Tab in the form), `docs/tui.md`.

## Out of scope

Single-line inline popups; tags anywhere but the form.

## Evidence

Tests: `Tab` after `ta` in tags opens the picker seeded `ta`, Enter inserts; `Tab` on an empty tags field still moves to the next field; known tags rank first. A `yaks tui --headless` frame, attached. Gate `cargo test --workspace`.
