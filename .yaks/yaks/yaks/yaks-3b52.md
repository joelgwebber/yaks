---
id: yaks-3b52
title: 'Session note timestamps: name and time zone of reviewed:'
status: shaving
type: task
priority: 2
created: '2026-11-06T21:36:37Z'
updated: '2026-11-10T17:50:27Z'
parent: yaks-3f35
tags:
- doks
- design
---

From yaks-3f35, item 2. A session note's `reviewed:` field holds the time the note was written (`sessions::build` in the `upkeep` work sets it to now), not when anyone reviewed it; the review stamps `promoted:`. And the note's path and `day:` use local time while `reviewed:` is UTC, so a note written at 23:30 in California sits under one day and claims another.

The name came from Hearth's `session-review` files, where a server reviews a conversation after the fact; for a note an agent writes about its own session, it reads as the review time. `default_since` (the start of the next `touched` window) reads it.

## To decide

- The field's name: keep `reviewed:` (Hearth's notes and `import-hearth` read alike), or rename it (`written:` / `saved:`) and read `reviewed:` as a fallback for imported Hearth notes.
- The time zone: write it with the local offset (`2026-11-06T13:30:00-08:00`), so it agrees with the path and `day:`, or keep UTC and derive the path from UTC too.

## Evidence

Tests for the chosen name and zone, the `yaks-upkeep` skill's description of the frontmatter, and `docs/`.

---
▸ 2026-11-06T21:37:48Z [schism-feedback]
kind: ask
needs: human
---

asked: needs human

Two details of the session note's time field, my lean on each.

1. **Name.** Rename `reviewed:` to `written:` for notes `yaks session note` writes, and keep reading `reviewed:` for Hearth's imported notes (where a server did review the conversation). Alternative: keep `reviewed:` so every session file reads alike.
2. **Zone.** Write it with the local offset (`2026-11-06T13:30:00-08:00`), so it agrees with the path and `day:`, which stay local (the day a person thinks the session was). Alternative: UTC everywhere, path included.

---
▸ 2026-11-06T21:47:25Z [sched-1]
kind: answer
needs: agent
---

1. Let's keep it simple, dropping anything hearth-import related (that was a very short-lived need). Cf yaks-ff4c.
2. Let's do UTC everywhere -- we have a presentation layer to reflect local timezones.

---
▸ 2026-11-06T21:49:23Z [sched-1]
kind: request
---

requested: needs agent

Let's also go ahead and knock out ff4c while we're doing this, for simplicity and clarity.

---
▸ 2026-11-10T17:49:48Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:27Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: delegating to worker sesstime-1 (Sonnet): rename the field to written: (old reviewed: read as fallback for existing notes), UTC everywhere including path and day:.
