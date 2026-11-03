---
id: yaks-c215
title: 'TUI phase 6b: one search register for every text subject'
status: shaving
type: task
priority: 2
created: '2026-11-03T03:46:32Z'
updated: '2026-11-03T05:14:49Z'
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
