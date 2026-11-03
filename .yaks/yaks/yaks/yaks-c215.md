---
id: yaks-c215
title: 'TUI phase 6b: one search register for every text subject'
status: shorn
type: task
priority: 2
created: '2026-11-03T03:46:32Z'
updated: '2026-11-03T05:26:23Z'
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

---
▸ 2026-11-03T05:24:48Z [find-1]

Headless frames (target/debug/yaks tui --headless --size 110x16 on a scratch farm: l, /cache, Enter, n, n, n). Frame 4: footer 'match 1/3 "cache"' after /cache Enter; 5 and 6: n steps to 2/3, 3/3; frame 7: the third n moves past the last match to the next matching yak (yak-8270), 'match 1/2'.

---
▸ 2026-11-03T05:25:48Z [find-1]

Decision: `LineNav.matched` is `Option<usize>` (None until `n`/`/` lands on a match), not a bare index. Rejected: `usize` starting at 0, because a yak opened with the register already set would show match 1 as current and the first `n` would skip it; "from none, `n` = first match, `N` = last" is the vim behaviour.
Decision: one find box is `Overlay::Find(SearchBox)`, used for both the yak detail and the open page (it picks the subject by `attach()`). Rejected: keeping the viewer's own `find_box` field next to an overlay; the two copies of the key routing were the duplication.
Decision: Esc in the pane (after a selection) still peels the register (`register = None`), as it peeled the find before; Esc in the find box restores the previous register. Rejected: Esc never clearing it (vim `@/`), which would leave no way to drop highlights short of an empty `/` Enter, and changes the old peel order.
Decision: `/` seeds the box with the register (cursor at the front, like the list's box), so `/` Enter keeps/re-lands on the current match as the old tests expect. Rejected: an empty box per `/` (vim), which breaks that and `Esc restores`.
Decision: `n`/`N` past the matches of a yak go to the next/previous row whose yak passes `tree::query_matches` (the list query's rule); `N` lands on that yak's LAST match, `n` on its first. Rejected: `N` also landing on the first match. Past the last matching yak it stays and notifies `no more matches for "q"` (no wrap in the detail).
Decision: `n`/`N` also put the line cursor on the match row (find_step sets `line` and `scroll`); before they scrolled only. Needed for "landing" on a match in the next yak and so Enter/Tab act on it.
Decision: list `n`/`N` are `bind(List, ...)` for every row kind; on a Wiki page row `N` stays WikiNew (a kind binding beats the any-kind one), so only `n` re-applies there. With an empty register: notification `nothing to find: / sets the search`. With a query set: nothing, as drafted.
Decision: the footer is the status bar: `match i/n "q"` is prefixed to the usual hint/link text (`match 0/n` before a match is current; `0/0` with none) in the yak detail and the viewer. Rejected: a new pane row, which would change the viewport arithmetic of both panes.

---
▸ 2026-11-03T05:26:14Z [find-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-03T05:26:23Z [find-1]

Done: one search register (`App.register`) for the yak detail and the text viewer; `LineNav.find` gone (it keeps `matched: Option<usize>`); `Overlay::DetailFind` + the viewer's `find_box` are `Overlay::Find`; J/K, link-follow, o/i keep the register; detail `n`/`N` hop to the next/previous matching yak; viewer `n` wraps; list `n`/`N` with no query apply the register; status bar shows `match i/n "q"`.
Evidence: `env -u YAKS_DIR -u YAKS_ACTOR cargo test --workspace` green (909 lib tests, all integration suites ok); `yaks verify yaks-c215` PASS; headless frames attached (find-register-frames.txt). New tests in src/tui/tests.rs (register_* / n_past_the_last... / viewers_n_wraps / detail_footer / list n) and linenav::find_matches_cycle. Docs: docs/tui.md (detail pane, viewer, Search & filter, new "The search register" paragraph), `?` lines in keymap.rs (one snapshot updated: help_overlay_shows_the_farm_keys).
Docs-parity grep: `grep -rn "detail_find|DetailFind|find_box|doc_find|open_detail_find|detail\.find|nav\.find\b" src docs .agents README.md` shows only test names; the skills and README do not describe the pane find keys.

---
▸ 2026-11-03T05:26:23Z [find-1]
kind: move
from: shaving
to: shorn
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAECO6JIBxBCtY3f3KUhDW5IL-PWT3Nbm
---

moved: shaving -> shorn
