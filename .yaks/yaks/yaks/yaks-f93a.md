---
id: yaks-f93a
title: Starred doks
status: shaving
type: feature
priority: 3
created: '2026-11-10T19:06:01Z'
updated: '2026-11-10T21:29:46Z'
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
