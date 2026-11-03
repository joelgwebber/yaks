---
id: yaks-947a
title: Implement the unified TUI model (/wiki/design/tui-model.md)
status: shaving
type: feature
priority: 2
created: '2026-11-03T03:46:30Z'
updated: '2026-11-03T05:08:57Z'
tags:
- tui
---

Build the agreed design, [the unified TUI model](/wiki/design/tui-model.md), in its phases. Each phase is a child yak that ships on its own and deletes what it replaces.

It implements these yaks, which are sheared as their phases land:

- yaks-953f: phases 1 and 6a.
- yaks-1bbd: phase 6b.
- yaks-c7ba: phase 6a.
- yaks-f1e6: phase 3.
- yaks-158d: phase 5.
- yaks-0ac1: phases 2 and 7.

Coordinated by herd-tui on a Delta cloud runner; workers are Sonnet 5.5 subthreads.

---
▸ 2026-11-03T03:47:40Z [herd-tui]
kind: move
from: hairy
to: shaving
child: yaks-cc16
thread: ksQQAxHGlH5JRk2_BHzLBG1rrJLOAC-kJZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T03:52:22Z [herd-tui]

Wave 1 started (herd-tui, base `9a8c7b4`, Sonnet 5.5 workers):

- `notify-1` on yaks-cc16 (phase 1), thread `ksQQAxHGlH5JRk2_BHzLBG1rrJLOAD9uc5IBxBCtY3f3KUhDW5IL-PWT3Nbm`.
- `keys-1` on yaks-fbb0 (phase 2), thread `ksQQAxHGlH5JRk2_BHzLBG1rrJLOAD9wu5IBxBCtY3f3KUhDW5IL-PWT3Nbm`.

Runner setup: rustup stable, and git 2.47.1 built into `~/.local` (the system git 2.39 lacks `worktree add --orphan` and `merge-tree --merge-base`, failing 22 tests and `yaks land`).

---
▸ 2026-11-03T04:11:59Z [herd-tui]

Wave 1 landed: phase 1 as `f18b13db` plus herd follow-up `a22c9843`, phase 2 as `642be42e`. Gate green on each.

Phase 3 is split into yaks-be65 (row kinds, Wiki as a view) and yaks-c354 (Sheds as a tree view), run back to back. Started `rows-1` (Sonnet 5.5) on yaks-be65 from `642be42e`, thread `ksQQAxHGlH5JRk2_BHzLBG1rrJLOAD_EipIBxBCtY3f3KUhDW5IL-PWT3Nbm`.

---
▸ 2026-11-03T04:27:44Z [herd-tui]

Phase 3a landed as `50af0a9a`. Started `sheds-2` on yaks-c354 (phase 3b) from it, thread `ksQQAxHGlH5JRk2_BHzLBG1rrJLOAD_xRZIBxBCtY3f3KUhDW5IL-PWT3Nbm`.

---
▸ 2026-11-03T05:08:57Z [herd-tui]

Progress (herd-tui):

- Phase 3 done: 3a `50af0a9a`, 3b `200d48fa` (yaks-5d08, yaks-f1e6 shorn).
- Phase 6a landed as `cd753ad8`, phase 4a as `a6634e38` (run in parallel, clean merge). Gate green on the combined tree.
- Started `nav-2` on yaks-9dec (phase 4b, the `LineNav` fold), thread `ksQQAxHGlH5JRk2_BHzLBG1rrJLOAECAE5IBxBCtY3f3KUhDW5IL-PWT3Nbm`. Next: 6b (yaks-c215), then 5 (yaks-a2c5), then 7 (yaks-6919).
- Disk: worker `target/` dirs are about 3.3 GB each on a 32 GB disk; landed workers' targets are deleted after landing.
