---
id: yaks-ef82
title: Human-friendly timestamp rendering
status: shorn
type: feature
priority: 3
created: '2026-08-27T16:15:00Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-faf5
needs: agent
---

In header fields like created/updated; but especially comment timestamps, which are currently raw RFC/UTC.
Bonus points for rendering in the local timezone.

---
▸ 2026-10-17T04:26:52Z [delta-lead]

Scope agreed 2026-10-17 (yaks-e9de): relative shorthand for nearby times ("5m ago", "3h ago", "yesterday"), an absolute local-time date beyond about a week, with the timezone shown where absolute; used for comment headers (yaks-93cd) and the Created/Updated fields. "Now" must be injectable (an `App` clock, a fixed instant in tests) so insta snapshots and docshots stay deterministic. Today `humanize_date` (src/tui/detail.rs) formats UTC with no zone label.

---
▸ 2026-10-17T04:31:30Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T04:31:30Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **dates-1**.
Scope: a pure formatter (new small module, e.g. src/tui/when.rs, or beside `humanize_date` in src/tui/detail.rs) taking the timestamp, a "now" and a timezone offset, used for the Created/Updated fields. Rules: under 1 min "just now"; under 1 h "Nm ago"; under 24 h "Nh ago"; yesterday "yesterday HH:MM"; under 7 days "Weekday HH:MM"; else "Oct 5, 2026 22:15" in local time; absolute forms carry the zone abbreviation or offset. Unparseable input passes through unchanged. Inject "now" + offset through `detail::build` (or an App-level clock) so tests and the headless/docshot paths are deterministic: no `set_var`, no reading the real clock in tests. Only `humanize_date`/`opt_date` and the Created/Updated lines in detail.rs: other workers own `scan_body_links` and the `Status:` line. Expose the formatter for reuse by comment headers (yaks-93cd, not yours).
Evidence: unit tests over the table of rules with a fixed now/offset; headless frame attached. Judge: coordinator at landing.
Do NOT regenerate docs/assets docshots (several workers would collide on the SVGs; the coordinator regenerates them after landing). Re-accept any insta snapshot your change moves (`INSTA_UPDATE=always cargo test`, or `cargo insta accept` if installed) and read every changed .snap diff before committing. `cargo fmt` reformats files nobody touched (src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs): run `cargo fmt -- <your files>` and `git checkout` those three if they changed. Docs/help parity per AGENTS.md (docs/tui.md, help_content in src/tui/render.rs) where behaviour a user sees changes.

---
▸ 2026-10-17T04:32:53Z [dates-1]

Start Sat Oct 17 04:32:53 AM UTC 2026; path: /home/joel/src/yaks/.delta/worktrees/9y0cpwz8ykh1/yaks/.yaks/yaks-ef82.md

[frame](yaks-ef82/frame.txt)

---
▸ 2026-10-17T04:37:33Z [dates-1]

Decision: formatter is `tui::when::{Clock, format_in}` (new src/tui/when.rs), not beside humanize_date, so comment headers (yaks-93cd) can reuse `app.clock.format(iso)`. `humanize_date` removed (its one caller moved).
Decision: injection via an `App.clock: when::Clock` field (enum `System` | `Fixed(now, offset)`) and `detail::build_at(.., &Clock)`; rejected a new parameter on `detail::build` for every caller/test. `build` survives only under `#[cfg(test)]` as a wrapper on the pinned test clock (2026-09-10 12:00 UTC, shown in UTC), so snapshots are deterministic with no set_var; no insta snapshot moved (all snapshot Created/Updated are `-`). `App::new` picks `Clock::app_default()` = System in the binary, Fixed under cfg(test). The headless `--headless` binary path therefore shows real local time (nothing deterministic to pin there; docshots run under cfg(test) so use the Fixed clock).
Decision: "under 24h" wins over "yesterday": 23h ago prints "23h ago", and "yesterday HH:MM" starts at exactly 24h (calendar day = yesterday in the display tz). 2..=6 calendar days back = weekday name; 7+ = absolute. Rejected strict calendar "yesterday" for 1h-old timestamps after midnight.
Decision: future timestamps beyond 60s of skew render absolute (not "in 5m"); within 60s = "just now".
Decision: zone label is `UTC` when offset is 0, else `+HH:MM` (chrono FixedOffset display), not a tz abbreviation (chrono has no tzdb; Local gives only an offset). Relative forms carry no zone.
Evidence: when.rs unit tests (rules table, offset-calendar days, Clock) + detail test `created_and_updated_use_the_clock`; headless frame attached (frame.txt: `Created: Aug 27, 2026 12:15 -04:00`, `Updated: 3h ago`, local tz -04:00 on this machine).
Docs: docs/tui.md detail-pane paragraph. No new key, so help_content unchanged; README does not describe the fields.
Summary: Created/Updated in the detail pane now render relative/local with labelled zone through a pure injectable formatter.

---
▸ 2026-10-17T04:37:42Z [dates-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T04:38:43Z [dates-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-ef82` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-ef82 --drop-request`, which removes this request note, so no new note is left behind.
