---
id: yaks-8d6a
title: Label auto-completion
status: shorn
type: feature
priority: 3
created: '2026-08-26T13:24:15Z'
updated: '2026-11-10T21:46:29Z'
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

---
▸ 2026-11-10T21:13:43Z [tagcomplete-1]

Start Tue Nov 10 21:13:43 UTC 2026; checkout /Users/joel/src/yaks/.delta/worktrees/sh9cspt26zn3/yaks

[tagframes](yaks-8d6a/tagframes.txt "Headless: Tab after 'tui, r' in the form's tags field opens the tag picker (known tag tui with description, then rust by use); Enter yields 'tui, rust'")

---
▸ 2026-11-10T21:46:21Z [tagcomplete-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T21:46:29Z [tagcomplete-1]

Decision: reuse the one `FuzzyPick`/`InsertRef` picker (new `tags` rows field and `CompleteKind::Tag`) rather than a second picker type or an inline popup.

Decision: partial tag = the run since the last comma or whitespace (not `is_ref_char`, which would split `c++` or `Foo`); none, so Tab moves on, when empty or right after a comma/space.

Decision: rows are known tags (schema order, description as text) then used tags by count (then name), ranked by prefix < substring < text-substring with a stable sort, so known tags win ties; rejected: ranking by count only, which would bury described tags.

Decision: the picker reads `schemas::load(root, YAK_TYPE).tags` itself in `App::tag_choices`; nothing added to `src/schemas.rs`.

Shape: `tag_completion` (editor.rs), `is_tags_row` (create.rs), Tab hook in `handle_create_key`, `tag_choices`/`open_ref_picker`/`finish_insert`/`commit_fuzzy` (handlers.rs), `tag_candidates`/`render_tag_results` (fuzzy.rs), hint on the status line (render.rs). Help: keymap/editor.rs Tab row, docs/tui.md. Tests in src/tui/tests.rs (+ snapshot); headless frame attached. Gate: cargo test --workspace passes, fmt clean, clippy count unchanged (94/98 before and after).

---
▸ 2026-11-10T21:46:29Z [tagcomplete-1]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFVYeZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
