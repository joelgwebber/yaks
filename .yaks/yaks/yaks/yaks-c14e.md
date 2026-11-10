---
id: yaks-c14e
title: Filtering out 'shorn'... doesn't
status: shaving
type: bug
priority: 3
created: '2026-11-10T19:08:25Z'
updated: '2026-11-10T21:35:04Z'
tags:
- tui
---

See the attached screenshot.

![bad-shorn-filter](yaks-c14e/bad-shorn-filter.png)

---
▸ 2026-11-10T21:11:50Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:12:23Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `shornfilter-1`, Sonnet 5.5.

## Scope

On the All view (a flat view, `view::ALL_KEY`), turning `shorn` off in the filter drawer leaves shorn yaks in the list (screenshot). Find why: the drawer's status chips against `flat_rows`/`view_count` in `src/tui/viewmodel.rs` and the drawer in `src/tui/drawer.rs` (the All view's spec lists every status; the drawer shows `hairy shaving` lit and `shorn dead` unlit, yet rows stay). Fix the rule so the live filter's statuses are what a flat view shows, on every flat view (All, Recent, custom sorted), and the tab count agrees with the rows.

## Out of scope

Tree views, unless the same rule is broken there too (then say so in a note and fix it the same way).

## Evidence

A test that fails first: All view, drawer toggles shorn off, no shorn row remains and the count matches. A `yaks tui --headless` frame of it, attached. Gate `cargo test --workspace`.

---
▸ 2026-11-10T21:13:10Z [shornfilter-1]

Start 2026-11-10T21:13:10Z; checkout /Users/joel/src/yaks/.delta/worktrees/pfapv7at78cw/yaks, base 686d328a.

[shorn-after](yaks-c14e/shorn-after.txt "After the fix: All view, drawer status chips hairy+shaving+dead lit (shorn off): 3 rows, no shorn yak (yak-1fc5), where before it stayed")
