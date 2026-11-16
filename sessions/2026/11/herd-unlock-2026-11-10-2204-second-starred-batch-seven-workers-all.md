---
type: session-review
title: 'Second starred batch: seven workers, all landed'
source: session-note
actor: herd-unlock
person: Joel Webber
host: joel
harness: delta
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-10
written: '2026-11-10T22:04:22Z'
good: 1
---

## Summary

Landed the first batch on `main` (`1a3388a8..686d328a`), then took Joel's second starred set (`yaks-464b`, `yaks-c14e`, `yaks-f04d`, `yaks-8d6a`, `yaks-f93a`, `yaks-6a05`) and pulled in three more that keep the gate and landings honest (`yaks-717b` with `yaks-06cd`, `yaks-760d`). Seven Sonnet workers ran in parallel; every one landed cleanly with no cherry-pick conflicts. herd-unlock-2 rewrote [/doks/design/knowledge-store.md](/doks/design/knowledge-store.md) and wrote this farm's known tags (`schemas/yak.md`). Batch record: [the landing note](yaks-1c89).

## Turns

- good: "land these changes on main, then move on to the next set of starred yaks"

## Saved

- yaks-06cd: Load-sensitive tests: init_modes worktree test and sync's private-farm test fail under parallel builds (hairy -> shorn)
  - [landed: 709ec3c5 yaks-717b: unique, cleaned-up temp dirs in init_modes and fa...](yaks-06cd#2026-11-10T22:01:13Z)
  - [Closed by herd-unlock-2 from flaky-1's work in yaks-717b (landed `709ec3c5`):](yaks-06cd#2026-11-10T22:03:35Z)
- yaks-1c89: Upkeep: session notes, session review and housekeeping (Hearth's knowledge procedures in yaks)
  - [Batch 2 by herd-unlock-2, 2026-11-10 (Joel's second starred set plus three ga...](yaks-1c89#2026-11-10T22:03:47Z)
- yaks-464b: Refresh /doks/design/knowledge-store.md: it still describes wiki/, yaks page and import-hearth (hairy -> shorn)
  - [Done (herd-unlock-2): /doks/design/knowledge-store.md rewritten to the curren...](yaks-464b#2026-11-10T21:14:45Z)
- yaks-6a05: TUI visibly slower when typing / navigating with many yaks visible (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `perf-1`, Sonnet 5.5.](yaks-6a05#2026-11-10T21:12:24Z)
  - [landed: e0005025 yaks-6a05: hoist is_multi_herd out of the per-row list rende...](yaks-6a05#2026-11-10T21:45:25Z)
- yaks-717b: tests/init_modes: farm_works_from_a_git_worktree... fails on a rerun (leftover share2-web-wt) (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `flaky-1`, Sonnet 5.5. Covers yaks-06cd...](yaks-717b#2026-11-10T21:12:25Z)
  - [landed: 709ec3c5 yaks-717b: unique, cleaned-up temp dirs in init_modes and fa...](yaks-717b#2026-11-10T22:01:13Z)
- yaks-760d: yaks land: record the landing after a conflicting cherry-pick is resolved (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `landcont-1`, Sonnet 5.5.](yaks-760d#2026-11-10T21:12:25Z)
  - [landed: f67e29fd yaks-760d: yaks land --continue and --abort finish or give u...](yaks-760d#2026-11-10T21:43:43Z)
- yaks-8d6a: Label auto-completion (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `tagcomplete-1`, Sonnet 5.5. (Depends o...](yaks-8d6a#2026-11-10T21:12:25Z)
  - [landed: ef5c259e yaks-8d6a: complete tags with Tab in the form's tags field](yaks-8d6a#2026-11-10T21:46:47Z)
- yaks-bcd7: Unknown tags: create and update say when a tag is not among the farm's known tags (created, hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `tagwarn-1`, Sonnet 5.5.](yaks-bcd7#2026-11-10T21:12:26Z)
  - [landed: de313d74 yaks-bcd7: warn when create/update/bulk write a tag not amon...](yaks-bcd7#2026-11-10T21:31:01Z)
- yaks-c14e: Filtering out 'shorn'... doesn't (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `shornfilter-1`, Sonnet 5.5.](yaks-c14e#2026-11-10T21:12:23Z)
  - [landed: d147c2bc yaks-c14e: flat views honour the drawer's status chips (Filt...](yaks-c14e#2026-11-10T21:43:15Z)
- yaks-f04d: Known labels in configuration (hairy -> shorn)
  - [herd-unlock-2, 2026-11-10: the "predefined labels with descriptions" this ask...](yaks-f04d#2026-11-10T21:15:14Z)
  - [Done (herd-unlock-2): known tags with descriptions live in schemas/yak.md (th...](yaks-f04d#2026-11-10T22:03:46Z)
- yaks-f93a: Starred doks (hairy -> shaving)
  - [Claim (yakherd herd-unlock-2): worker `dokstar-1`, Sonnet 5.5.](yaks-f93a#2026-11-10T21:12:24Z)
  - [landed: dd0656cc yaks-f93a: star doks with * (Doks tab, viewer), listed in th...](yaks-f93a#2026-11-10T21:38:08Z)
- changed [/doks/design/knowledge-store.md](/doks/design/knowledge-store.md): The farm as a knowledge store

## Lessons

- Seven parallel workers landed with zero conflicts this time: briefs named each worker's files and told the overlapping pair (shorn filter, latency) to keep to caching versus the rule. Worth keeping as the template.
- I deleted the farm's `schema` version file thinking it stray (it holds `8`); caught before commit and restored. Know a file before removing it, even in the farm.
- Checking a CLI change against the real farm created a real yak (`yaks-8532`), reverted with `yaks undo`. Check against a copy (`cp -R "$(yaks path)"` plus `YAKS_DIR`), as the TUI checks did.
- Workers found `yaks verify` refusing until a `verify:` is set on the yak when the config default doesn't apply as they expected; the brief should say `yaks verify` falls back to the config default.
- `farmsync` tests failed for dokstar-1 under seven parallel builds; flaky-1's fix landed after, and the final gate was green.
