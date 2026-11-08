---
id: yaks-ac6a
title: Recurring datetimes keep their wall-clock time across DST; a farm time zone
status: shaving
type: bug
priority: 2
created: '2026-11-07T16:57:32Z'
updated: '2026-11-08T00:52:45Z'
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

---
▸ 2026-11-07T17:04:37Z [herd-stars]
kind: ask
needs: human
---

asked: needs human

Three calls before this is built (herd-stars), my lean on each:

1. **Field name.** `timezone: America/New_York` in `.yaks/config.yaml` (IANA name). Alternative: `tz:`. Lean: `timezone`.
2. **Per herd?** Lean: farm-wide only, no `herds:` override. A family spread over zones keeps working, since a datetime `due:` keeps the offset it was given; only "today", `+2d` and date-only dues use the zone.
3. **Where the zone rules come from** (a new dependency either way, as plain `chrono` has no IANA database):
   - **`chrono-tz`**, the tz database compiled into the binary (roughly +0.5–1 MB; works the same on every platform, Windows included). Lean.
   - **The system's zoneinfo** (`/usr/share/zoneinfo`, via a small crate such as `tz-rs`): no size cost, but nothing on Windows and results vary with the OS's tzdata.
   - **`jiff`**, a newer date crate that does both: a second date library beside `chrono`.

Absent `timezone:`, the machine's zone, as today.

---
▸ 2026-11-08T00:48:13Z [sched-1]
kind: answer
needs: agent
---

1. `timezone:` works for me.
2. Agreed, per-herd would just get silly.
3. Yes, let's use the embedded one. The last thing we need is more environmental things to go wrong.

---
▸ 2026-11-08T00:49:15Z [herd-stars]
kind: pickup
---

picked up

herd-stars: building it as answered: farm-wide `timezone:`, chrono-tz embedded.

---
▸ 2026-11-08T00:49:17Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-08T00:52:09Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-tz`, per Joel's answers (2026-11-08): `timezone:` in `.yaks/config.yaml`, farm-wide only, the tz database embedded (`chrono-tz`).

**Scope.**

- `store::Config` reads `timezone:` (an IANA name). A name `chrono-tz` doesn't know: a warning (doctor, and once on load), then the machine's zone. Absent: the machine's zone, as today.
- One farm clock (a small module or `recur.rs` functions) used everywhere yaks today uses `Local` for dates: `today`, `+2d`/`+1w`, date-only `due:`, session note day and file name, display of datetimes.
- A datetime `due:` keeps the offset it was written with (no rewrite to `Z`); `every` steps on the wall clock in the farm zone (19:30 stays 19:30; `P1D` across a change is 23 or 25 hours), month-end clamping as today.
- `Cargo.toml`/`Cargo.lock`: add `chrono-tz` (allowed for this yak only). Measure the release binary delta and note it.
- Docs: `docs/cli.md` (config, due/every), `README.md` config block, the `yaks-scheduling` skill if it states the UTC behaviour.

**Evidence.** Tests: the description's repro (19:30 New York across 2026-11-01) fails first; a daily and a weekly yak across both US transitions; a monthly one on the 31st keeps clamping; a bad zone warns. Gate `cargo test --workspace`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-08T00:52:45Z [stars-tz]

Start (stars-tz): date -u Sun Nov  8 00:52:35 UTC 2026

- farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-ac6a.md
- top-level: /Users/joel/src/yaks/.delta/worktrees/ena37215nmvk/yaks
- HEAD: a29187b1de3d733b778ddc7f9fcc55df29b01458
