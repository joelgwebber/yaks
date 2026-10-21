---
id: yaks-e61e
title: 'TUI startup is quadratic in farm size (10k yaks: seconds; 50k: up to a minute)'
status: hairy
type: bug
priority: 1
created: '2026-10-18T20:27:09Z'
updated: '2026-10-18T20:27:09Z'
labels:
- tui
- perf
---

Found by scout scale-1 measuring yaks-b32e (flat layout at scale). TUI first frame (headless, `printf 'quit\n' | yaks tui --headless`), synthetic farms of realistic yak files: 1k 24 ms, 10k 2.1 s, 50k 17-57 s; fitted exponent climbs 1.21 -> 1.46 -> 1.80 -> 2.08. Almost all USER CPU (50k: 17.1 s user vs 0.33 s sys), so it is yaks' own algorithm, not the filesystem: `yaks list` is linear (51 ms / 202 ms / 770 ms) and so are ls, rg and git.

Reproduced by the yakherd with ~/go/bin/yaks on the scout's farms (this machine, warm): 10k 8.6 s; 20k 10.9 s; 20k with no id mentions in bodies 18.5 s (so mention scanning is not the cause, or not alone). The scout's farms are at /var/tmp/yakscale/{f1k,f10k,f50k,f20k_*} (variants: bare, noparent, nomention) until Joel releases them.

Likely suspects to profile first (perf/flamegraph on the 10k farm): per-row work that scans all tasks (tree building, family scope, `live_child_count`, blocked/`resolved_ids` per row, the memo/viewmodel cache keys), and the startup sheds scan. Evidence for the fix: a timing test or bench on a generated 10k farm that fails before (e.g. > 1 s) and passes after, plus the 50k first frame on this machine.
