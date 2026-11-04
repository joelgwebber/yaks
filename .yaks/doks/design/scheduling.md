---
type: topic
title: 'Scheduling: ticks, recurring yaks and watchers'
tags:
- design
- cli
- skills
---

# Scheduling: ticks, recurring yaks and watchers

Tracked in yaks-7fa6. Written after Joel's `review-watch` thread (2026-11-04), where a small skill that mirrors GitHub review requests into yaks kept reaching for launchd plists and `osascript`, because yaks gave it nowhere else to put "wake me" and "show me what is due".

## What exists

- `due:`, `every:` and `regrown:` fields, `list --due`, an `overdue` mark, and `yaks regrow-due` (yaks-2fc5, [the earlier design](/doks/design/knowledge-store.md) put the clock outside yaks).
- The TUI runs `regrow-due` on load and on each sync tick, so recurring yaks reopen while a TUI is open. Nothing else does.
- The inbox (`needs: human|agent`, `ask`, `answer`, `request`, `pickup`, `wait`) is the one queue for "someone must act".

## What the thread hit

- The TUI shows `Every:` and `Due:` but cannot set either.
- No agent-side way to run a scheduling round, so the skill grew its own `tick`.
- A regrown yak is only hairy. Nothing tells an agent it has work.
- No single view of "what is due or overdue", so the skill added a notification.
- Idempotent intake needed find-by-`source:` across all states (also found in [Haks](/doks/research/haks-orchestration-layer.md)).
- Nobody could tell from the docs or skills what the blessed mechanism was.

## Principles

1. **The human kicks it off.** `yaks tick`, by hand, or by a persistent agent later. yaks never installs a clock and never runs a command from a farm file: a synced farm is shared input, so a command registry in it would run other people's text.
2. **Use the inbox.** A recurring yak that needs someone goes to `needs:` like any other work. No second queue.
3. **The recurring yak is the registry.** Its description is the instruction; its notes are the history. A skill registers itself by creating one.
4. **One report.** `tick` ends by saying what it did and what needs you, so you can trust it without reading the farm.
5. **Durations only.** `P1W` and `+2d`, no weekday words.

## The pieces

| Piece | Yak |
|---|---|
| Relative `--due +2d`, and `yaks snooze` | yaks-0079 |
| `create --key <source-url>`, `list --source` | yaks-bdae |
| Regrow routes to the inbox | yaks-a041 (one open ask: how a yak remembers who to wake) |
| `yaks tick` | yaks-5e8d |
| TUI: set `due`/`every`, snooze key | yaks-2986 |
| TUI: Due tab and overdue chip | yaks-f561 |
| Skill `yaks-scheduling`, plus the edits to `yaks`, `yaks-working`, `yaks-coordinating`, docs | yaks-b569 |
| Acceptance: shrink `review-watch` onto it | yaks-79a6 |

## `yaks tick`

Reopens what is due (the same `Farm::regrow_due` the TUI calls), then prints:

- what regrew, and what was held (unreadable `every:`);
- overdue yaks, and those due within `--within` (default 3d);
- the inbox: awaiting you, awaiting an agent;
- stale claims;
- a one-line `yaks check` count.

`--dry-run` writes nothing. `--json` is the shape a harness or persistent agent reads. It does not sync unless `--sync` is passed; the two-machine case (both regrow the same yak, a duplicate `regrow` note) needs a test before `--sync` is recommended.

## Watcher contract (for the skill)

A custom skill that mirrors an outside queue into yaks must:

- find or create by `source:` (`create --key`), including shorn and dead yaks, so it never duplicates and never resurrects what the human buried;
- stamp its own actor;
- clear on a real signal from the outside, regrow on a re-request, and leave a hand-shorn yak shorn;
- never sync, never install a clock, never post an OS notification: `tick` is how the human hears;
- print one summary line, and exit non-zero only on failure;
- register by creating one recurring yak whose description says "run the scan, triage what it made".

## Open

- How a recurring yak remembers who to wake: see the ask on yaks-a041.
- Persistent agents: later, outside yaks. They should need only `tick --json` on a clock and `wait --for agent`.
