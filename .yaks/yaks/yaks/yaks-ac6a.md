---
id: yaks-ac6a
title: Recurring datetimes keep their wall-clock time across DST; a farm time zone
status: shorn
type: bug
priority: 2
created: '2026-11-07T16:57:32Z'
updated: '2026-11-08T01:16:19Z'
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

---
▸ 2026-11-08T01:01:38Z [stars-tz]

Progress (stars-tz), 2026-11-08 ~01:40Z: code compiles and the existing suite compiles; writing the new tests next.

Design as built:

- `src/zone.rs`: `Zone` = `Machine` | `Named(chrono_tz::Tz)` | `Fixed(FixedOffset)` (Fixed for tests/docshots only). `at`, `resolve` (wall clock to instant: repeated hour takes the first, a skipped hour moves on by the gap), `date_of`, `today`, `midnight`.
- `store::Config.timezone` (farm-wide, no `herds:` override), `Config::zone()`, `timezone_problem()`, `store::timezone_notice`; `Farm::zone()`.
- `recur::When` is now `Day(NaiveDate) | At(DateTime<FixedOffset>)`: a datetime keeps its offset on write; `Every::after/latest`, `plan`, `is_overdue`, `due_mark`, `due_rows` take the zone. `recur::today()` is gone: `zone.today()`.
- Zone passed to `sessions::build`, `tick::build` (Digest.zone), the `json::*` task renderers, `main` row renderers, and the TUI `Clock` (`System(Zone)`, `in_zone`, `zone`, `today`; `with_farm` sets it from config).

Local audit (`grep -rn Local src`): the only `chrono::Local` uses were `recur::today`, `sessions::build`, `main.rs` session-promote day and `tui/when.rs` (Clock::System format); all four moved to the farm zone. Every other grep hit is `Link::Local`/`Target::Local`/`Check::LocalCheckout`, unrelated. `Zone::Machine` itself is the one remaining `Local` (the fallback, by design).

Left alone on purpose (UTC by their own contract, not `Local`; follow-up): `check.rs` stale_after `today` (UTC date), `sources.rs` source dir date (UTC), `store::parse_since` date = midnight UTC (documented), machine-written stamps (`regrown:`, `created:`, notes) stay `Z`.

[ac6a-repro-and-size](yaks-ac6a/ac6a-repro-and-size.txt)

---
▸ 2026-11-08T01:10:37Z [stars-tz]

Decision notes (stars-tz):

- Decision: `When` is an enum `Day(NaiveDate) | At(DateTime<FixedOffset>)`, not a struct with a UTC instant. A day has no zone of its own, so its instant (`instant`, `deadline`) takes the farm zone at use; an instant keeps the offset it was written with. Rejected: resolving a day to a UTC instant at parse (needs the zone at every parse site) and a global "farm zone" static (not testable without ambient state).
- Decision: the zone is passed explicitly (`Zone` is `Copy`): `recur::{plan,is_overdue,due_mark,due_rows}`, `sessions::build`, `tick::build` (`Digest.zone`), `json::*` task renderers, and the TUI `Clock` (`System(Zone)`). Rejected: `OnceLock`/env, which the brief rules out for tests.
- Decision: `every` steps on the wall clock of the due instant viewed in the farm zone, counted from the anchor (not step by step), then resolved: a repeated hour takes the first, a skipped hour moves on by the gap; the next day returns to the original wall time because each step counts from the anchor. Rejected: chrono's `DateTime + Days`, which returns None in a gap/ambiguity.
- Decision: the report's own repro (`--due 2026-11-01T19:30:00-04:00`) is an instant that is 18:30 EST (clocks went back at 02:00 on Nov 1), so it correctly keeps 18:30 local; the tests use `2026-10-31T19:30:00-04:00` for the 19:30 case and assert the as-typed one too. The instant given is the truth; yaks does not guess that a wrong offset was meant.
- Decision: `timezone` is read as `Config.timezone: Option<String>` with `zone()`/`timezone_problem()`; an unknown name warns once per command on stderr (inside the existing no-`--json` block next to `min_version`), is a `doctor` issue `bad-timezone`, and the machine's zone is used. The TUI does not add a second warning. No `herds:` override.
- Decision: machine-written stamps (`regrown:`, `created:`, note headers, `now_iso`) stay UTC `Z`; only calendar dates use the farm zone. `check`'s `stale_after` today, `sources` dir date and `parse_since` dates stay UTC (not `Local`; follow-up if wanted).
- Decision: integration tests in `tests/recur.rs` now create their farms with `timezone: UTC` (`Farm::new`), because date-only dues and `+Nd` otherwise follow the machine's zone and the old UTC-date assertions were machine- and hour-dependent (they would fail right now, 00:xx UTC on a US machine). The TUI's `day_in` test helper uses the pinned test clock instead of `Local::now()`: forms now ask the app's clock for "today".
- Decision: `Zone::Fixed(FixedOffset)` exists for tests and the TUI's fixed test clock (`Clock::Fixed` keeps its offset; `in_zone` only changes `System`).

Size: release binary 11,114,464 -> 12,320,672 bytes (+1,206,208, about +1.15 MiB) with `chrono-tz` default tz database (`default-features = false`; pulls `phf 0.12` as a second phf version). Both built with `cargo build --release` (strip, thin LTO); the baseline from a `git archive` of HEAD in /tmp. Trimming needs chrono-tz's `filter-by-regex` (build-time), not done.

Friction: none blocking; `yaks note` is `yaks update --note`.

---
▸ 2026-11-08T01:12:29Z [stars-tz]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-08T01:12:39Z [stars-tz]

Summary (stars-tz): farm-wide `timezone:` (IANA, embedded chrono-tz) is the one farm clock for today, `+Nd`, date-only `due:`, regrow-due/tick, session note day and name, and datetime display (CLI and TUI); datetime `due:` keeps its offset; `every` steps on the wall clock in the farm zone. Bad zone: stderr warning once per command + `doctor` `bad-timezone`, machine zone used. Docs/help updated (docs/cli.md, docs/tui.md, README config block, yaks and yaks-scheduling skills, `--due`/`--every` clap help). Gate `cargo test --workspace`: all green (1087 unit + integration); `yaks verify` PASS. Old-vs-new repro and size attached.

---
▸ 2026-11-08T01:12:40Z [stars-tz]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEy3PZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-08T01:13:19Z [herd-stars]
kind: landed
commit: 913e4f4a8e9f28e0d5eae5aea363545604bdf633
source: cd5faac7e788d9d9190bb5a4d1121466cbe00097
branch: main
---

landed: 913e4f4a yaks-ac6a: farm time zone (timezone: + chrono-tz); recurring datetimes keep wall-clock time across DST

---
▸ 2026-11-08T01:16:19Z [herd-stars]

herd-stars: landed `913e4f4a` (worker `cd5faac7`, clean squash); gate `cargo test --workspace` green on the merged tree. Reproduced with my build on a scratch farm (`timezone: America/New_York`, daily from `2026-10-31T19:30:00-04:00`, shear, `regrow-due`): `due: 2026-11-08T19:30:00-05:00`. A bad zone (`Mars/Olympus`) warns once and uses the machine's. Binary +1.15 MiB (chrono-tz). Follow-ups the worker named (UTC by contract, not `Local`): `check` stale_after's today, source directory dates, `--since` dates.
