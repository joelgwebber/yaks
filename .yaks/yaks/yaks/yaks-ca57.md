---
id: yaks-ca57
title: Auto-complete for dok-links
status: shaving
type: feature
priority: 3
created: '2026-11-10T17:37:34Z'
updated: '2026-11-10T17:57:24Z'
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

[yaks__tui__tests__dok_picker_over_editor](yaks-ca57/yaks__tui__tests__dok_picker_over_editor.snap "Dok-link picker open over the comment editor after Tab on /doks/des (insta snapshot, 72x14)")

---
▸ 2026-11-10T17:57:21Z [doklink-1]

Decision: reuse FuzzyPick (new optional `doks` field, `dok_candidates` ranking) and FuzzyAction::InsertRef rather than adding a FuzzyAction::InsertDok variant or a new overlay: the commit/return-to-editor path is identical, so one `finish_insert` serves both. Rejected: a second picker type.

---
▸ 2026-11-10T17:57:22Z [doklink-1]

Decision: completion_context now returns a `Completion { len, seed, kind: Yak|Dok }` instead of a tuple; a dok link is tested first (its last segment may look like a yak ref), yak behaviour is unchanged.

---
▸ 2026-11-10T17:57:22Z [doklink-1]

Decision: the dok token is its own scan (`path_before_cursor`: alnum, - _ . /) rather than widening `is_ref_char`, which other code uses for yak refs. The trigger is only `/doks/` or `/<herd>/doks/` with a non-reserved herd segment (`store::RESERVED_HERD_NAMES`); `#`, relative `../doks`, `/sources/...` do not trigger.

---
▸ 2026-11-10T17:57:23Z [doklink-1]

Decision: the picker lists only DocGroup::Dok entries from `App.doks.entries` (reloaded via `reload_doks` if the Doks tab was never shown); ranking path-prefix < path-substring < title-substring, capped at 20 like the yak picker. Inserted text is `/<path>` including `.md`, no anchor.

---
▸ 2026-11-10T17:57:24Z [doklink-1]

Decision: the help-bar hint reads `Tab → complete dok link` in a dok context; yak hint unchanged. Docs: docs/tui.md (editor sentence + new bullet in Following links), keymap/editor.rs Tab rows (3). The yak skill and README do not mention Tab completion, so no change there (grep below).
