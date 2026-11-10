---
id: yaks-f93a
title: Starred doks
status: shaving
type: feature
priority: 3
created: '2026-11-10T19:06:01Z'
updated: '2026-11-10T21:37:40Z'
tags:
- doks
- tui
---

Would be useful to be able to star doks, just like yaks.

---
▸ 2026-11-10T21:11:50Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:12:24Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `dokstar-1`, Sonnet 5.5.

## Scope

`*` stars a dok the way it stars a yak: in the Doks tab (row) and with a dok open in the viewer. Stars live in the same per-user working set (`views_store` `working_set.json`, `toggle_working_set`); a dok is keyed by its farm path (`doks/...`), which never collides with a yak id. The Starred view lists starred doks after starred yaks (as the All view lists documents after yaks), and Enter opens them. Stale entries (a dok that moved or was deleted) are skipped, as missing yak ids are today. Bindings in `src/tui/keymap.rs` (`BINDINGS`, help lines), `docs/tui.md`.

## Out of scope

Sources, session notes; a CLI for stars.

## Evidence

Tests: star a dok in the Doks tab, it appears in Starred, `*` again removes it; the file round-trips both kinds. A `yaks tui --headless` frame of the Starred view with a dok, attached. Gate `cargo test --workspace`.

---
▸ 2026-11-10T21:13:32Z [dokstar-1]

Start 2026-11-10T21:13:29Z; checkout /Users/joel/src/yaks/.delta/worktrees/pcjanhhk4b1n/yaks

[starred](yaks-f93a/starred.txt "Starred view with two starred doks (headless frame, scratch farm): ⭐ ends each row")

[viewer](yaks-f93a/viewer.txt "A starred dok open in the viewer: ⭐ in the header")

---
▸ 2026-11-10T21:37:14Z [dokstar-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T21:37:40Z [dokstar-1]

Decision: one `*` command (`yak.star`) with a `Doks` row binding and a `Viewer` binding, not a new `dok.star` command. Rejected: a separate `Cmd` (a second name for one intent; `?` groups and the exceptions test already cover `YakStar`).

Decision: a dok is keyed by its farm path in the existing `working_set.json` `ids` list. Rejected: a second list or a file-format bump (a `doks/...` path never collides with a yak id, so the file stays as is).

Decision: only doks (`DocGroup::Dok`) can be starred; a source or session note answers "only a yak or a dok can be starred". Rejected: starring all documents (out of scope per the claim; sources have no stable single path: `source.md`).

Decision: in the Starred view yaks come first, then doks, each in star order. Rejected: one interleaved star order (the working set mixes both, but the All view's yaks-then-documents rule is the precedent, and the doks need the `Doc` row kind so keys dispatch right).

Decision: `view_count` for Starred now counts `working_set_rows().len()`, one rule for count and rows.

Decision: `*` with a dok open stars that dok (viewer binding), found by `AttachView::target` when it is a dok path; an attachment or another farm file says it cannot be starred. Rejected: silent no-op.

Decision: the star glyph ⭐ ends a dok row (path column shrinks by 3 cells) and follows the kind label in the viewer header, as on yak rows and the yak pane header.
