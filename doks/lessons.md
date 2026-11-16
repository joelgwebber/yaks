---
type: topic
title: 'Lessons: how to work in this project'
created: '2026-11-10T17:56:26Z'
tags:
- skills
- meta
description: 'Read at every session start: short rules learned across sessions, kept by the weekly reflection round.'
---

# Lessons: how to work in this project

Kept by the weekly reflection round (the `yaks-upkeep` skill, "Reflecting on sessions"); read at session start. Budget: about 50 lines.

## Rules

- Parallel workers that each add to one shared doc section or dispatch arm conflict at every landing: land them as one serial arc, or give the section to one worker. ([detail](#landing-parallel-workers))
- Run a new listing or command on the real farm, and rasterize and look at regenerated docshots: tests and green gates missed both. ([detail](#look-at-the-real-artifact))
- Load-sensitive tests fail under parallel builds; rerun alone before calling a worker's gate red. ([detail](#flaky-gates-under-load))

## Landing parallel workers

- **Rule:** symbol-scoped briefs don't prevent conflicts in a shared line: `docs/tui.md`'s key table, `docs/cli.md`, `run_cmd`'s no-op arm, `src/main.rs` dispatch. Land such workers in a serial arc.
- **How we know:** cherry-pick conflicts at every landing, both times ([session](/sessions/2026/11/herd-stars-2026-11-07-2025-starred-batch-13-yaks-shorn-with-sonnet.md), [session](/sessions/2026/11/herd-upkeep-2-2026-11-06-1904-wrapping-up-yaks-1c89-tools-refs.md)).
- **Seen:** 2026-11-07

## Look at the real artifact

- **Rule:** after changing a listing, run it on the real farm; after regenerating docshots, rasterize (`yaks render`) and look at each.
- **How we know:** the first real `session touched` run showed noisy gists the tests passed ([session](/sessions/2026/11/herd-upkeep-2026-11-06-0008-upkeep-in-yaks-assessment-note-links-ses.md)); a key change made docshot scenes shoot an empty list with nothing failing ([session](/sessions/2026/11/herd-stars-2026-11-07-2025-starred-batch-13-yaks-shorn-with-sonnet.md)).
- **Seen:** 2026-11-07

## Flaky gates under load

- **Rule:** a red gate under parallel builds is first rerun alone (`cargo test --no-fail-fast`); the known offenders are yaks-06cd and yaks-717b.
- **How we know:** `init_modes` and `sync` tests failed for workers in two sessions ([session](/sessions/2026/11/herd-upkeep-2026-11-06-0008-upkeep-in-yaks-assessment-note-links-ses.md), [session](/sessions/2026/11/herd-upkeep-2-2026-11-06-1904-wrapping-up-yaks-1c89-tools-refs.md)).
- **Seen:** 2026-11-06
