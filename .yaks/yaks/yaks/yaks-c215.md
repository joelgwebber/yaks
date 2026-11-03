---
id: yaks-c215
title: 'TUI phase 6b: one search register for every text subject'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:32Z'
updated: '2026-11-03T05:24:48Z'
parent: yaks-947a
depends_on:
- yaks-aac1
tags:
- tui
---

Phase 6, the pane half, of [the design](/wiki/design/tui-model.md#4-search) (yaks-1bbd): `/` in the pane sets one search register (replacing `detail_find` and the viewer's find), highlighted in every text subject, kept across `J`/`K` and `o`/`i`; `n` past the last match moves to the next matching row; `n`/`N` in a list re-apply it as the query.

---
▸ 2026-11-03T05:08:29Z [herd-tui]

Claim draft (yakherd herd-tui), to start after phase 4b (yaks-9dec) lands: worker `find-1`.

**Task.** One search register for every text subject ([design §4](/wiki/design/tui-model.md#4-search)).

- `App.register: Option<String>` (vim's `@/`). `/` in the yak detail or the text viewer edits it, not the list query; `Esc` restores the previous register.
- `LineNav.find` stops being per-page state: every text subject reads `App.register` for highlights and `n`/`N`, and `LineNav` keeps only the current match index.
- The yak detail and the viewer both highlight matches of the register as soon as they open. `J`/`K`, link-follow and `o`/`i` keep the register and reset only the match index.
- `n` past the last match in the yak detail moves to the next row whose yak matches the register (the list query's matching rule), landing on its first match; `N` likewise backwards. In the viewer `n` wraps.
- In a list, `n`/`N` with an empty query re-apply the register as the query; with a query they do nothing (lean from the earlier design; record it).
- The detail footer shows `match 2/5 "q"` while the register is set.
- `Overlay::DetailFind` and the viewer's own `find_box` become one find box (`SearchBox`) that edits the register.

---
▸ 2026-11-03T05:14:49Z [herd-tui]
kind: move
from: hairy
to: shaving
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T05:14:49Z [herd-tui]

herd-tui: the draft above is the claim. Worker `find-1` (Sonnet 5.5), base main `e9e3c6c2` (4b landed: `App.detail: LineNav`). Out of scope: history stacks (phase 5), keys other than `/`, `n`, `N`, Esc. Evidence: the gate; tests for each bullet (register kept across `J`/`K` and `o`/`i`; `n` past the last match moves to the next matching row; viewer `n` wraps; list `n` with no query re-applies the register; footer shows `match i/n`); docs/tui.md and the `?` lines; a headless frame of the footer. Judge: herd-tui at landing.

---
▸ 2026-11-03T05:23:47Z [find-1]

Start 2026-11-03T05:23:42Z. Checkout /home/user/yaks-9, base main e9e3c6c2, farm file /home/user/.local/share/yaks/farms/yaks-b0f8dcf9/.yaks/yaks/yaks/yaks-c215.md.

Progress: implemented the register (App.register, Overlay::Find replacing DetailFind and the viewer's find_box, LineNav.find gone, matched: Option<usize>), n/N row hop in the detail, list n/N, status-bar chip. Tests next, then docs and the headless frame.

[find-register-frames](yaks-c215/find-register-frames.txt)
