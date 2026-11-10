---
id: yaks-c14e
title: Filtering out 'shorn'... doesn't
status: shorn
type: bug
priority: 3
created: '2026-11-10T19:08:25Z'
updated: '2026-11-10T21:42:50Z'
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

---
▸ 2026-11-10T21:35:05Z [shornfilter-1]

Cause: `flat_rows` (and `view_count`) only read the status scope for `dead` (`want_dead = statuses.contains(Dead)`); every other status was ignored, so on All (a flat view whose spec lists every status) switching `shorn` off in the drawer changed nothing.

Fix: one rule, `FilterSpec::admits_status` (`src/filter.rs`): the listed statuses, or with none listed every status but dead. `flat_rows` uses the live filter's, `view_count` the view spec's. Tree views were not broken (`tree::build` already honours `spec.statuses`).

Tests (failed first, `cargo test --bin yaks`): `the_drawer_status_chips_scope_flat_views`, `a_flat_views_count_follows_its_statuses` in `tui/tests.rs`.

Decision: put the rule on `FilterSpec` as `admits_status`, not inline in viewmodel. Rejected: patching both call sites inline (two copies of the rule, which is how they drifted); changing `filter::apply` (it already has the rule but also takes `include_dead` and parent scope, a heavier fit for the latency worker's `flat_rows`).

Decision: the tab count (`view_count`) uses the view's saved spec statuses, not the live filter, as before (documented "independent of the live facets"); only the rule changed. So the All tab count does not move when a chip is toggled, as for every other facet.

Decision: docs: one sentence under Filter drawer in `docs/tui.md`. Grep for old wording ("dead ones included", "flat view", statuses) in docs/ .agents/skills/ README.md src/ found nothing contradicting it.

---
▸ 2026-11-10T21:42:44Z [shornfilter-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T21:42:49Z [shornfilter-1]

Summary: flat views now honour every status chip via FilterSpec::admits_status (src/filter.rs, used by flat_rows and view_count); docs/tui.md updated; verify PASS; headless frame attached.

---
▸ 2026-11-10T21:42:50Z [shornfilter-1]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFVPepIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
