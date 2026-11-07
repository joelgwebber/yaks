---
id: yaks-ac6a
title: Recurring datetimes keep their wall-clock time across DST; a farm time zone
status: hairy
type: bug
priority: 2
created: '2026-11-07T16:57:32Z'
updated: '2026-11-07T16:57:32Z'
parent: yaks-44b3
tags:
- cli
---

A recurring yak with a datetime `due:` drifts by an hour across a daylight-saving change, because `recur.rs` stores and adds in UTC (`Every::after` on `DateTime<Utc>`). Found while designing Hearth's reminders on a farm (hearth-1ed0, hearth-cfac in the Hearth repo's farm).

Repro on a scratch farm (2026-11-07):

```sh
yaks create "Bins out" --every P1D --due 2026-11-01T19:30:00-04:00 --wake human
yaks shorn <id>; yaks tick --json
# regrew: occurrence 2026-11-06T23:30:00Z, due 2026-11-07T23:30:00Z
```

The offset is dropped on write (`due: 2026-11-07T23:30:00Z`), so after US clocks went back on 2026-11-01 the reminder fires at 18:30 local, not 19:30.

Proposal:

- A farm time zone, `timezone: America/New_York` in `.yaks/config.yaml` (IANA name), used for `today`, `+2d`/`+1w`, date-only `due:` and the display of datetimes. Absent: the machine's zone, as today.
- A datetime `due:` keeps the offset it was given, and `every` steps in the farm's zone on the wall clock (19:30 stays 19:30), as calendars do. `P1D` across the change is 23 or 25 hours.
- Tests: a daily and a weekly yak across both US transitions; a month-end monthly one still keeps its day.

Needs a decision on the field name and whether the zone may be per herd (a family spread over zones). Lean: farm-wide only.
