---
id: yaks-ef82
title: Human-friendly timestamp rendering
status: shaving
type: feature
priority: 3
created: '2026-08-27T16:15:00Z'
updated: '2026-10-17T04:31:30Z'
parent: yaks-faf5
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
