---
id: yaks-0fa7
title: Backdate session notes and promotions for imported history
status: hairy
type: feature
priority: 2
created: '2026-11-06T21:36:36Z'
updated: '2026-11-06T21:37:28Z'
parent: yaks-3f35
tags:
- cli
- doks
---

From yaks-3f35, item 1. Importing a devlog as session notes (schism-5b88) needed each note dated to the day it describes, but `yaks session note` takes no date and `yaks session promote` always stamps today. The importer hand-edited `day`, `reviewed` and `promoted` and committed by hand (82 commits with a `Backdated: imported 2026-11-06` trailer).

## Wanted

- `yaks session note --at <time>` (RFC 3339 or a date): the path, `day` and the note's write time come from it, not the clock.
- `yaks session promote --on <day>`: stamps that day as `promoted:`.
- The farm commit each one makes is dated to that time too (author and committer date), with a trailer saying it was backdated and when, so `git log` order matches the history and nobody mistakes it for live work.
- A backdated note has no live session behind it: don't generate `## Saved` from `touched` (yaks-d11e); require the note to bring its own, or write `- nothing`.

## Related

yaks-80bd's "Backdate an import" bullet is the same need for sources and doks (Hearth's `HEARTH_DATE`). Decide whether one global `--date`/`YAKS_DATE` serves every write, or the session flags stay separate; lean: one mechanism, with the session flags as its first users. yaks-3b52 settles the name and zone of the time field this sets.

## Evidence

A test that a backdated note lands at the right path with the right fields and commit date, and the `yaks-upkeep` skill (plus `docs/cli.md`, `--help`) saying how to import history.
