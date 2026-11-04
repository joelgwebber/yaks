---
id: yaks-7fa6
title: 'Scheduling arc: ticks, recurring yaks that wake someone, and a skill for watchers'
status: shaving
type: epic
priority: 2
created: '2026-11-04T22:30:49Z'
updated: '2026-11-04T22:36:22Z'
tags:
- design
- cli
- tui
- skills
---

Finish the scheduling story that `due:`/`every:`/`regrow-due` (yaks-2fc5) started, so a person can kick off a scheduling round by hand, trust that it updates everything, and be shown what needs them, and so a custom skill such as `review-watch` stays small and on the rails.

Origin: Joel's review-watch thread (2026-11-04). The skill ended up reaching for launchd plists and `osascript` notifications because yaks gave it no way to wake an agent, no way to see "what is due" in one place, and no TUI way to set `due:`/`every:`. Design: [Scheduling: ticks, recurring yaks and watchers](/doks/design/scheduling.md).

## Shape

- **`yaks tick`** is the one hand-kicked command: reopen what is due, then report what needs you.
- **A recurring yak is a standing instruction.** On regrow it goes to the existing inbox (`needs: agent` or `needs: human`), so no second queue exists.
- **The recurring yak is the registry.** Its description says what to do; yaks runs no commands and installs no clocks.
- **Watchers follow a contract** (idempotent, keyed by `source:`, never sync, never schedule themselves) written down in a new `yaks-scheduling` skill.

## Out of scope

Persistent agents, launchd/cron integration and OS notifications. A persistent agent later runs `yaks tick --json` on a clock and `yaks wait --for agent`; nothing here should need to change for it.

## Decisions (Joel, 2026-11-04)

- Regrow wakes through the inbox, using existing constructs.
- One `yaks tick` command with `--dry-run`, not a separate read-only command.
- No command-hook registry.
- Relative dates (`+2d`, `+1w`) for `--due`; durations only, no weekday words.
- `create --key` and `list --source` belong in this arc.
- A new `yaks-scheduling` skill.

---
▸ 2026-11-04T22:36:22Z [sched-1]
kind: move
from: hairy
to: shaving
child: yaks-0079
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
